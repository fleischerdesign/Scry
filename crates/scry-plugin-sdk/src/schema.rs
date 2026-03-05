// Scry Semantic Schema - The standard vocabulary for all plugins and services.

pub mod traits {
    /// The primary representative image for an entity.
    pub const PHOTO: &str = "scry.visual/photo";

    /// The avatar of a user or person.
    pub const AVATAR: &str = "scry.core/avatar";

    /// The display name or title.
    pub const NAME: &str = "scry.core/name";

    /// A short description or bio.
    pub const DESCRIPTION: &str = "scry.core/description";

    /// Geographic location name.
    pub const CITY: &str = "scry.core/city";
}

pub mod predicates {
    /// Links an event to its subject (e.g. who did it).
    pub const SUBJECT: &str = "scry.core/subject";

    /// Links an event to a location.
    pub const LOCATION: &str = "scry.core/location";
}
