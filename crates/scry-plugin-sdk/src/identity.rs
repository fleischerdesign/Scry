use uuid::Uuid;

/// Generates a deterministic UUID v5 for any entity in the Scry ecosystem.
/// 
/// This function is domain-agnostic and ensures consistency by:
/// 1. Using a stable root namespace (scry.io).
/// 2. Normalizing all components (lowercase, trimmed).
/// 3. Joining components with a separator to prevent overlap.
pub fn create_id(namespace: &str, components: &[&str]) -> String {
    // The fixed root for all Scry identities
    let scry_root = Uuid::new_v5(&Uuid::NAMESPACE_DNS, b"scry.io");
    
    // Create a namespace-specific UUID (e.g., for "scry.music")
    let ns_uuid = Uuid::new_v5(&scry_root, namespace.as_bytes());
    
    // Normalize and join components
    let key = components.iter()
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("|");
        
    Uuid::new_v5(&ns_uuid, key.as_bytes()).to_string()
}
