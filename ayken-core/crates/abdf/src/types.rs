//! ABDF Type System
//!
//! Bu modül, ABDF formatında kullanılacak veri tiplerini
//! mantıksal (logical) seviyede tanımlar.
//!
//! # Örnek Kullanım
//!
//! ```
//! use abdf::types::{AbdfType, AbdfScalarType};
//!
//! let int_type = AbdfType::Scalar(AbdfScalarType::I32);
//! assert!(int_type.is_scalar());
//!
//! let vector_type = AbdfType::Vector(AbdfScalarType::F32);
//! assert!(vector_type.is_vector());
//!
//! let tensor_type = AbdfType::Tensor {
//!     base: AbdfScalarType::F32,
//!     rank: 2,
//! };
//! assert!(tensor_type.is_tensor());
//! ```

/// Temel scalar tipler.
/// Bu tipler hem tabular veriler hem de vektör/tensor verileri
/// için kullanılabilir.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum AbdfScalarType {
    /// 32-bit signed integer (örn: sayaçlar, id'ler)
    I32,
    /// 64-bit signed integer (örn: timestamp, büyük id'ler)
    I64,
    /// 32-bit floating point (örn: sensör verisi, yaklaşık değerler)
    F32,
    /// 64-bit floating point (örn: yüksek hassasiyetli hesaplar)
    F64,
    /// Boolean değer (true/false)
    Bool,
}

/// Yüksek seviyeli ABDF tipi.
/// Bu tipler, veri sütunlarının veya alanlarının ne tür veri taşıyacağını anlatır.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AbdfType {
    /// Tek bir scalar değer (örn: i32, f64, bool)
    Scalar(AbdfScalarType),

    /// UTF-8 string (metin) veri tipi
    Utf8,

    /// Tek boyutlu vektör (örn: embedding, zaman serisi)
    ///
    /// Örnek: `Vec<f32>`, `Vec<i32>`
    Vector(AbdfScalarType),

    /// Çok boyutlu tensor (örn: görüntü, matris, 3D/4D veri)
    ///
    /// `rank`: kaç boyutlu olduğunu belirtir (örn: 2 = matris)
    Tensor {
        base: AbdfScalarType,
        rank: u8,
    },
}

impl AbdfScalarType {
    /// Bu scalar tip sayısal mı? (bool hariç)
    pub fn is_numeric(&self) -> bool {
        matches!(self, Self::I32 | Self::I64 | Self::F32 | Self::F64)
    }

    /// Bu scalar tip float mı?
    pub fn is_float(&self) -> bool {
        matches!(self, Self::F32 | Self::F64)
    }
}

impl AbdfType {
    /// Bu tip scalar mı?
    pub fn is_scalar(&self) -> bool {
        matches!(self, AbdfType::Scalar(_))
    }

    /// Bu tip UTF-8 string mi?
    pub fn is_utf8(&self) -> bool {
        matches!(self, AbdfType::Utf8)
    }

    /// Bu tip vektör mü?
    pub fn is_vector(&self) -> bool {
        matches!(self, AbdfType::Vector(_))
    }

    /// Bu tip tensor mü?
    pub fn is_tensor(&self) -> bool {
        matches!(self, AbdfType::Tensor { .. })
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scalar_numeric_checks() {
        let t_i32 = AbdfScalarType::I32;
        let t_f64 = AbdfScalarType::F64;
        let t_bool = AbdfScalarType::Bool;

        assert!(t_i32.is_numeric());
        assert!(t_f64.is_numeric());
        assert!(!t_bool.is_numeric());

        assert!(!t_i32.is_float());
        assert!(t_f64.is_float());
        assert!(!t_bool.is_float());
    }

    #[test]
    fn abdf_type_kind_checks() {
        let t1 = AbdfType::Scalar(AbdfScalarType::I32);
        let t2 = AbdfType::Utf8;
        let t3 = AbdfType::Vector(AbdfScalarType::F32);
        let t4 = AbdfType::Tensor {
            base: AbdfScalarType::F32,
            rank: 2,
        };

        assert!(t1.is_scalar());
        assert!(!t1.is_utf8());
        assert!(!t1.is_vector());
        assert!(!t1.is_tensor());

        assert!(t2.is_utf8());
        assert!(!t2.is_scalar());

        assert!(t3.is_vector());
        assert!(!t3.is_tensor());

        assert!(t4.is_tensor());
        assert!(!t4.is_vector());
    }
}
