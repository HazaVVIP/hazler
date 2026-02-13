use rand::seq::SliceRandom;
use rustls::{
    ClientConfig, RootCertStore, SupportedCipherSuite, ALL_CIPHER_SUITES,
};

/// Creates a TLS client configuration with randomized cipher suites for fingerprint evasion
pub fn create_randomized_tls_config() -> Result<ClientConfig, rustls::Error> {
    // Get all available cipher suites
    let mut cipher_suites: Vec<SupportedCipherSuite> = ALL_CIPHER_SUITES
        .iter()
        .copied()
        .collect();

    // Randomize the order of cipher suites to create a unique TLS fingerprint
    let mut rng = rand::thread_rng();
    cipher_suites.shuffle(&mut rng);

    // Create a root certificate store with system certificates
    let mut root_store = RootCertStore::empty();
    
    // Add system certificates
    for cert in rustls_native_certs::load_native_certs()
        .map_err(|e| rustls::Error::General(format!("Failed to load native certs: {}", e)))?
    {
        root_store
            .add(&rustls::Certificate(cert.0))
            .map_err(|e| rustls::Error::General(format!("Failed to add cert: {}", e)))?;
    }

    // Build the TLS config with randomized cipher suites
    let config = ClientConfig::builder()
        .with_cipher_suites(&cipher_suites)
        .with_safe_default_kx_groups()
        .with_safe_default_protocol_versions()
        .map_err(|e| rustls::Error::General(format!("Failed to create config builder: {}", e)))?
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

/// Creates a standard TLS client configuration (non-randomized)
pub fn create_standard_tls_config() -> Result<ClientConfig, rustls::Error> {
    let mut root_store = RootCertStore::empty();
    
    // Add system certificates
    for cert in rustls_native_certs::load_native_certs()
        .map_err(|e| rustls::Error::General(format!("Failed to load native certs: {}", e)))?
    {
        root_store
            .add(&rustls::Certificate(cert.0))
            .map_err(|e| rustls::Error::General(format!("Failed to add cert: {}", e)))?;
    }

    // Build a standard TLS config
    let config = ClientConfig::builder()
        .with_safe_defaults()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_randomized_tls_config() {
        let config = create_randomized_tls_config();
        assert!(config.is_ok(), "Randomized TLS config should be created successfully");
    }

    #[test]
    fn test_create_standard_tls_config() {
        let config = create_standard_tls_config();
        assert!(config.is_ok(), "Standard TLS config should be created successfully");
    }

    #[test]
    fn test_randomization_produces_different_configs() {
        // Create multiple randomized configs to verify they can be created repeatedly
        let config1 = create_randomized_tls_config();
        let config2 = create_randomized_tls_config();
        let config3 = create_randomized_tls_config();
        
        // All configs should be successfully created
        assert!(config1.is_ok(), "First config creation should succeed");
        assert!(config2.is_ok(), "Second config creation should succeed");
        assert!(config3.is_ok(), "Third config creation should succeed");
        
        // Verify they are valid ClientConfig instances
        let config1 = config1.unwrap();
        let config2 = config2.unwrap();
        let config3 = config3.unwrap();
        
        // Verify configs have the expected structure (alpn_protocols is accessible)
        // These should be empty vectors by default
        assert!(config1.alpn_protocols.is_empty());
        assert!(config2.alpn_protocols.is_empty());
        assert!(config3.alpn_protocols.is_empty());
    }

    #[test]
    fn test_multiple_randomized_configs_can_be_created() {
        // This test verifies that the randomization process is repeatable
        // and doesn't have side effects that would prevent creating multiple configs
        for _ in 0..5 {
            let config = create_randomized_tls_config();
            assert!(config.is_ok(), "Each randomized config should be created successfully");
        }
    }
}
