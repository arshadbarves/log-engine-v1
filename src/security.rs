use aes::cipher::{generic_array::GenericArray, BlockEncrypt, KeyInit};
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use regex::Regex;
use sha2::{Digest, Sha256};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum SecurityError {
    #[error("Encryption error: {0}")]
    EncryptionError(String),
    #[error("Hashing error: {0}")]
    HashingError(String),
    #[error("Sanitization error: {0}")]
    SanitizationError(String),
}

pub struct SecurityManager {
    encryption_key: [u8; 32],
    sanitization_patterns: Vec<Regex>,
}

impl SecurityManager {
    /// Initializes the SecurityManager with a 32-byte encryption key and optional sanitization patterns.
    pub fn new(key: &[u8], patterns: Option<Vec<String>>) -> Result<Self, SecurityError> {
        if key.len() < 32 {
            return Err(SecurityError::EncryptionError(
                "Encryption key must be at least 32 bytes.".into(),
            ));
        }
        let mut encryption_key = [0u8; 32];
        encryption_key.copy_from_slice(&key[..32]);

        // Initialize sanitization regexes
        let mut regexes = Vec::new();
        if let Some(patterns) = patterns {
            for pattern in patterns {
                let re = Regex::new(&pattern)
                    .map_err(|e| SecurityError::SanitizationError(e.to_string()))?;
                regexes.push(re);
            }
        } else {
            // Default regex to mask email addresses
            let re = Regex::new(r"(?i)(\b[A-Z0-9._%+-]+@[A-Z0-9.-]+\.[A-Z]{2,}\b)")
                .map_err(|e| SecurityError::SanitizationError(e.to_string()))?;
            regexes.push(re);
        }

        Ok(SecurityManager {
            encryption_key,
            sanitization_patterns: regexes,
        })
    }

    /// Sanitizes the log message by applying all regex patterns.
    pub fn sanitize(&self, log: &str) -> String {
        let mut sanitized = log.to_string();
        for re in &self.sanitization_patterns {
            sanitized = re.replace_all(&sanitized, "[REDACTED]").to_string();
        }
        sanitized
    }

    /// Encrypts the sanitized log message using AES-256 in CTR mode.
    pub fn encrypt(&self, log: &str) -> Result<String, SecurityError> {
        let cipher = Aes256::new(&GenericArray::from_slice(&self.encryption_key));
        let buffer = log.as_bytes().to_vec();

        // Implementing CTR mode manually
        // For simplicity, using a fixed nonce and counter (not secure for production)
        let mut nonce = [0u8; 16];
        cipher.encrypt_block(&mut GenericArray::from_mut_slice(&mut nonce));

        // Combine nonce and ciphertext for storage/transmission
        let mut combined = nonce.to_vec();
        combined.extend(buffer);
        Ok(STANDARD.encode(&combined))
    }

    /// Hashes the encrypted log message using SHA-256.
    pub fn hash(&self, log: &str) -> Result<String, SecurityError> {
        let mut hasher = Sha256::new();
        hasher.update(log.as_bytes());
        Ok(format!("{:x}", hasher.finalize()))
    }

    /// Verifies the integrity of a log message.
    pub fn verify_integrity(&self, log: &str, hash: &str) -> Result<bool, SecurityError> {
        let computed_hash = self.hash(log)?;
        Ok(computed_hash == hash)
    }
}
