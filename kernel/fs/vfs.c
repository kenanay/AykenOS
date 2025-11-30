// kernel/fs/vfs.c
// Simple RAM-backed tarfs implementation for the early VFS layer.

#include <stddef.h>
#include <stdint.h>
#include <stdbool.h>

#include "../include/fs.h"
#include "../ai/ayken_core_lm_format.h"

#define TAR_BLOCK_SIZE 512
#define VFS_MAX_FILES  16
#define VFS_MAX_OPEN   16

typedef struct __attribute__((packed)) {
    char     name[100];
    char     mode[8];
    char     uid[8];
    char     gid[8];
    char     size[12];
    char     mtime[12];
    char     chksum[8];
    char     typeflag;
    char     linkname[100];
    char     magic[6];
    char     version[2];
    char     uname[32];
    char     gname[32];
    char     devmajor[8];
    char     devminor[8];
    char     prefix[155];
    char     pad[12];
} tar_header_t;

typedef struct {
    const char   *path;   // normalized path without leading slash
    const uint8_t *data;  // file contents
    uint64_t      size;   // file size in bytes
} vfs_node_t;

struct vfs_file {
    const vfs_node_t *node;
    uint64_t          offset;
    bool              in_use;
};

static vfs_node_t  g_ramfs_nodes[VFS_MAX_FILES];
static uint32_t    g_ramfs_count = 0;
static vfs_file_t  g_open_files[VFS_MAX_OPEN];

static uint8_t     g_initrd_tar[2048];
static uint64_t    g_initrd_tar_size = 0;

static uint8_t     g_dummy_model[256];

static void vfs_memset(void *dst, int value, uint64_t size)
{
    uint8_t *p = (uint8_t*)dst;
    for (uint64_t i = 0; i < size; i++) {
        p[i] = (uint8_t)value;
    }
}

static void vfs_memcpy(void *dst, const void *src, uint64_t size)
{
    uint8_t *d = (uint8_t*)dst;
    const uint8_t *s = (const uint8_t*)src;
    for (uint64_t i = 0; i < size; i++) {
        d[i] = s[i];
    }
}

static uint64_t vfs_strlen(const char *s)
{
    uint64_t len = 0;
    while (s && s[len]) {
        len++;
    }
    return len;
}

static int vfs_strcmp(const char *a, const char *b)
{
    while (*a && *b && *a == *b) {
        a++;
        b++;
    }
    return (int)((unsigned char)*a - (unsigned char)*b);
}

static const char* vfs_strip_leading_slashes(const char *path)
{
    while (path && *path == '/') {
        path++;
    }
    return path ? path : "";
}

static bool vfs_paths_match(const char *stored, const char *requested)
{
    const char *req = vfs_strip_leading_slashes(requested);
    const char *st  = vfs_strip_leading_slashes(stored);
    return vfs_strcmp(st, req) == 0;
}

static uint64_t tar_octal_to_u64(const char *str, size_t len)
{
    uint64_t value = 0;
    for (size_t i = 0; i < len && str[i]; i++) {
        if (str[i] < '0' || str[i] > '7') {
            break;
        }
        value = (value << 3) + (uint64_t)(str[i] - '0');
    }
    return value;
}

static void tar_write_octal(uint64_t value, char *out, size_t size)
{
    for (size_t i = 0; i < size; i++) {
        out[i] = '0';
    }

    size_t pos = size >= 2 ? size - 2 : 0;
    while (value > 0 && pos < size) {
        out[pos] = (char)('0' + (value & 7));
        value >>= 3;
        if (pos == 0) {
            break;
        }
        pos--;
    }

    if (size > 0) {
        out[size - 1] = '\0';
    }
}

static void tar_fill_checksum(tar_header_t *hdr)
{
    for (int i = 0; i < 8; i++) {
        hdr->chksum[i] = ' ';
    }

    uint32_t sum = 0;
    const uint8_t *bytes = (const uint8_t*)hdr;
    for (size_t i = 0; i < TAR_BLOCK_SIZE; i++) {
        sum += bytes[i];
    }

    tar_write_octal(sum, hdr->chksum, sizeof(hdr->chksum));
    hdr->chksum[sizeof(hdr->chksum) - 1] = ' ';
}

static uint64_t build_dummy_model(uint8_t *buffer, uint64_t buffer_size)
{
    const uint64_t weights_size = 64;

    ayken_lm_file_header_t hdr;
    vfs_memset(&hdr, 0, sizeof(hdr));

    vfs_memcpy(hdr.magic, AYKEN_LM_MAGIC, 7);
    hdr.version              = 1;
    hdr.quant_type           = AYKEN_QUANT_Q4_0;
    hdr.vocab_size           = 1;
    hdr.hidden_size          = 1;
    hdr.n_layers             = 1;
    hdr.n_heads              = 1;
    hdr.tensor_count         = 0;
    hdr.tensor_table_offset  = sizeof(hdr);
    hdr.weights_offset       = sizeof(hdr);
    hdr.weights_size         = weights_size;

    uint64_t total_size = hdr.weights_offset + hdr.weights_size;
    if (total_size > buffer_size) {
        return 0;
    }

    vfs_memset(buffer, 0, buffer_size);
    vfs_memcpy(buffer, &hdr, sizeof(hdr));

    for (uint64_t i = 0; i < weights_size; i++) {
        buffer[hdr.weights_offset + i] = (uint8_t)(0xA0 + (i & 0x0F));
    }

    return total_size;
}

static void build_initrd_tar(void)
{
    uint64_t model_size = build_dummy_model(g_dummy_model, sizeof(g_dummy_model));
    if (model_size == 0) {
        g_initrd_tar_size = 0;
        return;
    }

    tar_header_t hdr;
    vfs_memset(&hdr, 0, sizeof(hdr));

    const char *file_path = "system/aykencorelm/model.bin";
    uint64_t path_len = vfs_strlen(file_path);
    if (path_len >= sizeof(hdr.name)) {
        path_len = sizeof(hdr.name) - 1;
    }
    for (uint64_t i = 0; i < path_len; i++) {
        hdr.name[i] = file_path[i];
    }

    tar_write_octal(0644, hdr.mode, sizeof(hdr.mode));
    tar_write_octal(0, hdr.uid, sizeof(hdr.uid));
    tar_write_octal(0, hdr.gid, sizeof(hdr.gid));
    tar_write_octal(model_size, hdr.size, sizeof(hdr.size));
    tar_write_octal(0, hdr.mtime, sizeof(hdr.mtime));
    hdr.typeflag = '0';

    hdr.magic[0] = 'u'; hdr.magic[1] = 's'; hdr.magic[2] = 't';
    hdr.magic[3] = 'a'; hdr.magic[4] = 'r';
    hdr.magic[5] = '\0';
    hdr.version[0] = '0';
    hdr.version[1] = '0';

    tar_fill_checksum(&hdr);

    vfs_memset(g_initrd_tar, 0, sizeof(g_initrd_tar));
    vfs_memcpy(g_initrd_tar, &hdr, sizeof(hdr));

    uint64_t copy_size = model_size;
    vfs_memcpy(g_initrd_tar + TAR_BLOCK_SIZE, g_dummy_model, copy_size);

    uint64_t aligned_data = ((copy_size + TAR_BLOCK_SIZE - 1) / TAR_BLOCK_SIZE) * TAR_BLOCK_SIZE;
    g_initrd_tar_size = TAR_BLOCK_SIZE + aligned_data + (TAR_BLOCK_SIZE * 2);

    if (g_initrd_tar_size > sizeof(g_initrd_tar)) {
        g_initrd_tar_size = sizeof(g_initrd_tar);
    }
}

static void vfs_register_node(const char *path, const uint8_t *data, uint64_t size)
{
    if (g_ramfs_count >= VFS_MAX_FILES) {
        return;
    }

    g_ramfs_nodes[g_ramfs_count].path = path;
    g_ramfs_nodes[g_ramfs_count].data = data;
    g_ramfs_nodes[g_ramfs_count].size = size;
    g_ramfs_count++;
}

static void vfs_mount_initrd(const uint8_t *tar, uint64_t tar_size)
{
    g_ramfs_count = 0;

    uint64_t offset = 0;
    while (offset + TAR_BLOCK_SIZE <= tar_size) {
        const tar_header_t *hdr = (const tar_header_t *)(tar + offset);

        if (hdr->name[0] == '\0') {
            break;
        }

        uint64_t file_size = tar_octal_to_u64(hdr->size, sizeof(hdr->size));
        const uint8_t *file_data = tar + offset + TAR_BLOCK_SIZE;

        vfs_register_node(hdr->name, file_data, file_size);

        uint64_t advance = ((file_size + TAR_BLOCK_SIZE - 1) / TAR_BLOCK_SIZE) * TAR_BLOCK_SIZE;
        offset += TAR_BLOCK_SIZE + advance;
    }
}

void vfs_init(void)
{
    for (uint32_t i = 0; i < VFS_MAX_OPEN; i++) {
        g_open_files[i].in_use = false;
        g_open_files[i].node   = NULL;
        g_open_files[i].offset = 0;
    }

    build_initrd_tar();

    if (g_initrd_tar_size > 0) {
        vfs_mount_initrd(g_initrd_tar, g_initrd_tar_size);
    }
}

vfs_file_t *vfs_open(const char *path, vfs_mode_t mode)
{
    if (!path || mode != VFS_MODE_READ) {
        return NULL;
    }

    const vfs_node_t *target = NULL;
    for (uint32_t i = 0; i < g_ramfs_count; i++) {
        if (vfs_paths_match(g_ramfs_nodes[i].path, path)) {
            target = &g_ramfs_nodes[i];
            break;
        }
    }

    if (!target) {
        return NULL;
    }

    for (uint32_t i = 0; i < VFS_MAX_OPEN; i++) {
        if (!g_open_files[i].in_use) {
            g_open_files[i].in_use = true;
            g_open_files[i].node   = target;
            g_open_files[i].offset = 0;
            return &g_open_files[i];
        }
    }

    return NULL;
}

int vfs_read(vfs_file_t *file, void *buffer, uint64_t size)
{
    if (!file || !file->in_use || !file->node || !buffer) {
        return -1;
    }

    uint64_t remaining = (file->offset < file->node->size)
        ? (file->node->size - file->offset)
        : 0;

    uint64_t to_copy = (size < remaining) ? size : remaining;
    if (to_copy == 0) {
        return 0;
    }

    vfs_memcpy(buffer, file->node->data + file->offset, to_copy);
    file->offset += to_copy;

    return (int)to_copy;
}

int vfs_seek(vfs_file_t *file, int64_t offset, vfs_seek_whence_t whence)
{
    if (!file || !file->in_use || !file->node) {
        return -1;
    }

    int64_t base = 0;
    if (whence == VFS_SEEK_SET) {
        base = 0;
    } else if (whence == VFS_SEEK_CUR) {
        base = (int64_t)file->offset;
    } else if (whence == VFS_SEEK_END) {
        base = (int64_t)file->node->size;
    } else {
        return -1;
    }

    int64_t new_off = base + offset;
    if (new_off < 0 || (uint64_t)new_off > file->node->size) {
        return -1;
    }

    file->offset = (uint64_t)new_off;
    return 0;
}

int vfs_close(vfs_file_t *file)
{
    if (!file || !file->in_use) {
        return -1;
    }

    file->in_use = false;
    file->node   = NULL;
    file->offset = 0;
    return 0;
}
