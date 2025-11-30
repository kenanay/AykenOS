#include "../include/mm.h"              // paging_map, phys_alloc_frame
#include "../drivers/console/fb_console.h"
#include "ayken_core_lm.h"
#include "ayken_core_lm_format.h"
#include "../fs/vfs.h"                  // vfs_open, vfs_read, vfs_close (varsayım)

#define AYKEN_CORE_LM_BASE_VA  0xFFFFA00000000000ULL
#define AYKEN_CORE_LM_MAX_SIZE (64ULL * 1024 * 1024)  // max 64MB model

static lm_model_t core_model;

// Model dosyasının RAM'deki görüntüsü
static uint8_t *g_model_base   = NULL;  // header + tensor table + weights
static uint8_t *g_weights_base = NULL;  // sadece weights bloğunun başlangıcı

static ayken_lm_tensor_desc_t *g_tensors     = NULL;
static uint32_t                g_tensor_count = 0;

// virt tarafta ardışık bir bölge ayır + map et
static void* ayken_core_lm_map_region(uint64_t size_bytes)
{
    uint64_t pages = (size_bytes + PAGE_SIZE - 1) / PAGE_SIZE;

    uint64_t base_va = AYKEN_CORE_LM_BASE_VA;

    for (uint64_t i = 0; i < pages; i++) {
        uint64_t phys = phys_alloc_frame();
        if (!phys) {
            fb_print("[AykenCoreLM] phys_alloc_frame() FAILED!\n");
            return NULL;
        }

        uint64_t va = base_va + i * PAGE_SIZE;
        // user bit yok, sadece kernel: flags = 0
        paging_map(va, phys, 0);
    }

    return (void*)base_va;
}

void ayken_core_lm_init(void)
{
    fb_print("[AykenCoreLM] Initializing AI core...\n");

    vfs_file_t *f = vfs_open("/system/aykencorelm/model.bin", VFS_MODE_READ);
    if (!f) {
        fb_print("[AykenCoreLM] Failed to open model.bin\n");
        return;
    }

    // 1) Önce header'ı oku (stack üzerinde)
    ayken_lm_file_header_t hdr;
    int r = vfs_read(f, &hdr, sizeof(hdr));
    if (r != (int)sizeof(hdr)) {
        fb_print("[AykenCoreLM] Failed to read header.\n");
        vfs_close(f);
        return;
    }

    // 2) Magic / version / boyut kontrolleri
    if (__builtin_memcmp(hdr.magic, AYKEN_LM_MAGIC, 7) != 0) {
        fb_print("[AykenCoreLM] Invalid model magic.\n");
        vfs_close(f);
        return;
    }

    if (hdr.version != 1) {
        fb_print("[AykenCoreLM] Unsupported model version.\n");
        vfs_close(f);
        return;
    }

    // model dosyasının tamamı: weights_offset + weights_size
    uint64_t total_size = hdr.weights_offset + hdr.weights_size;
    if (total_size == 0 || total_size > AYKEN_CORE_LM_MAX_SIZE) {
        fb_print("[AykenCoreLM] Invalid or too large model size.\n");
        vfs_close(f);
        return;
    }

    fb_print("[AykenCoreLM] Header OK. Mapping model region...\n");

    // 3) Tüm model dosyası için sanal bölge ayır + map et
    void *base = ayken_core_lm_map_region(total_size);
    if (!base) {
        fb_print("[AykenCoreLM] Failed to map model region.\n");
        vfs_close(f);
        return;
    }

    g_model_base = (uint8_t*)base;

    // 4) Dosyanın başına geri dön ve tüm model.bin'i RAM'e oku
    if (vfs_seek(f, 0, VFS_SEEK_SET) < 0) {
        fb_print("[AykenCoreLM] vfs_seek(0) failed.\n");
        vfs_close(f);
        return;
    }

    int read_bytes = vfs_read(f, base, (int)total_size);
    if (read_bytes != (int)total_size) {
        fb_print("[AykenCoreLM] Failed to read full model.\n");
        vfs_close(f);
        return;
    }

    vfs_close(f);

    // 5) Header, tensor tablosu ve weights pointer'larını ayarla
    ayken_lm_file_header_t *phdr = (ayken_lm_file_header_t*)g_model_base;

    g_tensor_count = phdr->tensor_count;
    g_tensors = (ayken_lm_tensor_desc_t*)(
        g_model_base + phdr->tensor_table_offset
    );
    g_weights_base = g_model_base + phdr->weights_offset;

    // 6) core_model yapısını doldur
    core_model.vocab_size   = phdr->vocab_size;
    core_model.hidden_size  = phdr->hidden_size;
    core_model.n_layers     = phdr->n_layers;
    core_model.n_heads      = phdr->n_heads;
    core_model.quant_type   = (ayken_quant_type_t)phdr->quant_type;
    core_model.weights      = g_weights_base;         // sadece weights bloğu
    core_model.weights_size = phdr->weights_size;

    fb_print("[AykenCoreLM] Model loaded. tensors=");
    fb_print_hex(g_tensor_count);
    fb_print("\n");

    fb_print("[AykenCoreLM] Ready.\n");
}

lm_model_t* ayken_core_lm_get(void)
{
    return &core_model;
}

const ayken_lm_tensor_desc_t*
ayken_core_lm_find_tensor(ayken_tensor_type_t type, int layer_idx)
{
    if (!g_tensors || g_tensor_count == 0) {
        return NULL;
    }

    for (uint32_t i = 0; i < g_tensor_count; i++) {
        ayken_lm_tensor_desc_t *t = &g_tensors[i];

        if (t->type != (uint32_t)type) {
            continue;
        }

        // layer_idx == -1 ise "global tensor" (EMBED, W_LOGITS gibi)
        if ((int32_t)t->layer_idx == layer_idx ||
            (int32_t)t->layer_idx == -1) {
            return t;
        }
    }

    return NULL;
}

const void*
ayken_core_lm_get_tensor_data(const ayken_lm_tensor_desc_t *desc)
{
    if (!desc || !g_weights_base) {
        return NULL;
    }

    return (const void*)(g_weights_base + desc->offset);
}
