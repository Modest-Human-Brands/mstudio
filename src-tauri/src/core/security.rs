use serde::{Deserialize, Serialize};
use std::{ptr::null_mut, slice};

#[cfg(target_os = "windows")]
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::BOOL,
        Security::Cryptography::{
            CertCloseStore, CertEnumCertificatesInStore, CertNameToStrW, CertOpenStore,
            CryptAcquireCertificatePrivateKey, CryptCreateHash, CryptDestroyHash, CryptHashData,
            CryptReleaseContext, CryptSignHashW, NCryptFreeObject, NCryptSignHash, ALG_ID,
            BCRYPT_PKCS1_PADDING_INFO, CERT_CONTEXT, CERT_KEY_SPEC, CERT_OPEN_STORE_FLAGS,
            CERT_QUERY_ENCODING_TYPE, CERT_STORE_PROV_SYSTEM_W, CERT_STRING_TYPE,
            CERT_SYSTEM_STORE_CURRENT_USER, CERT_SYSTEM_STORE_LOCAL_MACHINE, CERT_X500_NAME_STR,
            CRYPT_ACQUIRE_ALLOW_NCRYPT_KEY_FLAG, HCRYPTPROV_OR_NCRYPT_KEY_HANDLE, NCRYPT_HANDLE,
            NCRYPT_KEY_HANDLE, NCRYPT_PAD_PKCS1_FLAG,
        },
    },
};

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CertItem {
    pub index: usize,
    pub subject: String,
    pub issuer: String,
    pub certificate_der_hex: String,
    pub certificate_chain_der_hex: Vec<String>,
}

#[cfg(target_os = "windows")]
struct CryptoHandleGuard {
    handle: HCRYPTPROV_OR_NCRYPT_KEY_HANDLE,
    key_spec: CERT_KEY_SPEC,
    caller_free: BOOL,
}

#[cfg(target_os = "windows")]
impl Drop for CryptoHandleGuard {
    fn drop(&mut self) {
        if self.caller_free.as_bool() && self.handle.0 != 0 {
            unsafe {
                if self.key_spec.0 == 0xFFFFFFFF || self.key_spec.0 == 0 {
                    let _ = NCryptFreeObject(NCRYPT_HANDLE(self.handle.0));
                } else {
                    let _ = CryptReleaseContext(self.handle.0, 0);
                }
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(target_os = "windows")]
unsafe fn get_cert_name(cert: &CERT_CONTEXT, name_type: CERT_STRING_TYPE) -> String {
    let len = CertNameToStrW(
        cert.dwCertEncodingType,
        &cert.pCertInfo.as_ref().unwrap().Subject,
        name_type,
        None,
    );
    if len <= 1 {
        return "Unknown Subject".to_string();
    }
    let mut buf = vec![0u16; len as usize];
    CertNameToStrW(
        cert.dwCertEncodingType,
        &cert.pCertInfo.as_ref().unwrap().Subject,
        name_type,
        Some(&mut buf),
    );
    String::from_utf16_lossy(&buf[..buf.len().saturating_sub(1)])
}

#[cfg(target_os = "windows")]
unsafe fn get_cert_name_issuer(cert: &CERT_CONTEXT, name_type: CERT_STRING_TYPE) -> String {
    let len = CertNameToStrW(
        cert.dwCertEncodingType,
        &cert.pCertInfo.as_ref().unwrap().Issuer,
        name_type,
        None,
    );
    if len <= 1 {
        return "Unknown Issuer".to_string();
    }
    let mut buf = vec![0u16; len as usize];
    CertNameToStrW(
        cert.dwCertEncodingType,
        &cert.pCertInfo.as_ref().unwrap().Issuer,
        name_type,
        Some(&mut buf),
    );
    String::from_utf16_lossy(&buf[..buf.len().saturating_sub(1)])
}

#[tauri::command]
pub async fn list_certificates() -> Result<Vec<CertItem>, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Hardware certificate signing is only supported on Windows OS.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let mut cert_list = Vec::new();
        unsafe {
            let store_name = to_wide("MY");
            let store_handle = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER),
                Some(store_name.as_ptr() as *const _),
            )
            .map_err(|e| format!("Failed to open Certificate Store: {}", e))?;

            let mut p_cert_context: *const CERT_CONTEXT = null_mut();
            let mut index = 0;

            loop {
                p_cert_context = CertEnumCertificatesInStore(store_handle, Some(p_cert_context));
                if p_cert_context.is_null() {
                    break;
                }

                let cert = &*p_cert_context;
                let subject = get_cert_name(cert, CERT_X500_NAME_STR);
                let issuer = get_cert_name_issuer(cert, CERT_X500_NAME_STR);

                let der_bytes =
                    slice::from_raw_parts(cert.pbCertEncoded, cert.cbCertEncoded as usize);
                let leaf_hex = hex::encode(der_bytes);

                let mut chain_hex = Vec::new();
                for store_type in &["CA", "ROOT"] {
                    for location in &[
                        CERT_SYSTEM_STORE_CURRENT_USER,
                        CERT_SYSTEM_STORE_LOCAL_MACHINE,
                    ] {
                        if let Ok(parent_store) = CertOpenStore(
                            CERT_STORE_PROV_SYSTEM_W,
                            CERT_QUERY_ENCODING_TYPE(0),
                            None,
                            CERT_OPEN_STORE_FLAGS(*location),
                            Some(to_wide(store_type).as_ptr() as *const _),
                        ) {
                            let mut p_parent: *const CERT_CONTEXT = null_mut();
                            loop {
                                p_parent =
                                    CertEnumCertificatesInStore(parent_store, Some(p_parent));
                                if p_parent.is_null() {
                                    break;
                                }
                                let p_cert = &*p_parent;
                                let parent_der = slice::from_raw_parts(
                                    p_cert.pbCertEncoded,
                                    p_cert.cbCertEncoded as usize,
                                );
                                chain_hex.push(hex::encode(parent_der));
                            }
                            let _ = CertCloseStore(parent_store, 0);
                        }
                    }
                }

                cert_list.push(CertItem {
                    index,
                    subject,
                    issuer,
                    certificate_der_hex: leaf_hex,
                    certificate_chain_der_hex: chain_hex,
                });
                index += 1;
            }
            let _ = CertCloseStore(store_handle, 0);
        }
        Ok(cert_list)
    }
}

#[tauri::command]
pub async fn sign_hash(digest_hex: String, cert_index: usize) -> Result<String, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("Hardware certificate signing is only supported on Windows OS.".to_string());
    }

    #[cfg(target_os = "windows")]
    {
        let digest = match hex::decode(&digest_hex) {
            Ok(d) if d.len() == 32 => d,
            _ => return Err("Expected a valid 32-byte SHA-256 hex digest.".to_string()),
        };

        unsafe {
            let store = CertOpenStore(
                CERT_STORE_PROV_SYSTEM_W,
                CERT_QUERY_ENCODING_TYPE(0),
                None,
                CERT_OPEN_STORE_FLAGS(CERT_SYSTEM_STORE_CURRENT_USER),
                Some(to_wide("MY").as_ptr() as *const _),
            )
            .map_err(|e| format!("Failed to open Windows Store: {}", e))?;

            let mut p_cert_context: *const CERT_CONTEXT = null_mut();
            let mut curr_idx = 0;
            let mut target_cert: *const CERT_CONTEXT = null_mut();

            loop {
                p_cert_context = CertEnumCertificatesInStore(store, Some(p_cert_context));
                if p_cert_context.is_null() {
                    break;
                }
                if curr_idx == cert_index {
                    target_cert = p_cert_context;
                    break;
                }
                curr_idx += 1;
            }

            if target_cert.is_null() {
                let _ = CertCloseStore(store, 0);
                return Err(format!("Certificate index {} not found.", cert_index));
            }

            let mut handle = HCRYPTPROV_OR_NCRYPT_KEY_HANDLE(0);
            let mut key_spec = CERT_KEY_SPEC(0);
            let mut caller_free = BOOL(0);

            let acquired = CryptAcquireCertificatePrivateKey(
                target_cert,
                CRYPT_ACQUIRE_ALLOW_NCRYPT_KEY_FLAG,
                None,
                &mut handle,
                Some(&mut key_spec),
                Some(&mut caller_free),
            );

            let _ = CertCloseStore(store, 0);

            if acquired.is_err() || handle.0 == 0 {
                return Err(
                    "Failed to acquire hardware token private key. Is the USB connected?"
                        .to_string(),
                );
            }

            let _guard = CryptoHandleGuard {
                handle,
                key_spec,
                caller_free,
            };

            let signature_bytes = if key_spec.0 == 0xFFFFFFFF || key_spec.0 == 0 {
                let alg_id = to_wide("SHA256");
                let padding_info = BCRYPT_PKCS1_PADDING_INFO {
                    pszAlgId: PCWSTR(alg_id.as_ptr()),
                };
                let mut sig_len = 0u32;

                NCryptSignHash(
                    NCRYPT_KEY_HANDLE(handle.0),
                    Some(&padding_info as *const _ as *const _),
                    &digest,
                    None,
                    &mut sig_len,
                    NCRYPT_PAD_PKCS1_FLAG,
                )
                .map_err(|e| format!("NCryptSignHash length estimation failed: {}", e))?;

                let mut sig_buffer = vec![0u8; sig_len as usize];
                NCryptSignHash(
                    NCRYPT_KEY_HANDLE(handle.0),
                    Some(&padding_info as *const _ as *const _),
                    &digest,
                    Some(&mut sig_buffer),
                    &mut sig_len,
                    NCRYPT_PAD_PKCS1_FLAG,
                )
                .map_err(|e| format!("NCryptSignHash execution failed: {}", e))?;

                sig_buffer
            } else {
                let mut h_hash = 0usize;
                CryptCreateHash(
                    handle.0,
                    ALG_ID(0x0000800C),
                    0,
                    0,
                    &mut h_hash as *mut _ as *mut _,
                )
                .map_err(|_| "Legacy CryptCreateHash failed.".to_string())?;

                if CryptHashData(h_hash, &digest, 0).is_err() {
                    let _ = CryptDestroyHash(h_hash);
                    return Err("Legacy CryptHashData failed.".to_string());
                }

                let mut sig_len = 0u32;
                if CryptSignHashW(h_hash, key_spec.0, None, 0, None, &mut sig_len).is_err() {
                    let _ = CryptDestroyHash(h_hash);
                    return Err("Legacy CryptSignHashW length estimation failed.".to_string());
                }

                let mut sig_buffer = vec![0u8; sig_len as usize];
                let ok = CryptSignHashW(
                    h_hash,
                    key_spec.0,
                    None,
                    0,
                    Some(sig_buffer.as_mut_ptr()),
                    &mut sig_len,
                );
                let _ = CryptDestroyHash(h_hash);

                if ok.is_err() {
                    return Err("Legacy CryptSignHashW execution failed.".to_string());
                }
                sig_buffer
            };

            Ok(hex::encode(signature_bytes))
        }
    }
}
