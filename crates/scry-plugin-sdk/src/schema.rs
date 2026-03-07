// Scry Semantic Schema - The standard vocabulary for all plugins and services.
// Oriented towards industry standards like schema.org but simplified for Scry.

pub mod namespaces {
    /// Core system identities (Users, Devices, Apps)
    pub const CORE: &str = "scry.core";
    /// Musical entities (Artists, Tracks, Albums)
    pub const MUSIC: &str = "scry.music";
    /// People, Contacts and Organizations
    pub const PERSON: &str = "scry.person";
    /// Physical locations, cities, and points of interest
    pub const PLACE: &str = "scry.place";
    /// Entertainment media (Movies, TV Shows, Books, Games)
    pub const MEDIA: &str = "scry.media";
    /// Health, Fitness and Biometrics
    pub const HEALTH: &str = "scry.health";
    /// Financial data (Transactions, Accounts, Assets)
    pub const FINANCE: &str = "scry.finance";
    /// Communication (Messages, Emails, Calls)
    pub const COMM: &str = "scry.comm";
    /// Environmental data (Weather, Sensors, Smart Home)
    pub const ENV: &str = "scry.env";
    /// Software, Tools and Services
    pub const SOFTWARE: &str = "scry.software";
}

pub mod traits {
    // --- General ---
    pub const NAME: &str = "scry.core/name";
    pub const DESCRIPTION: &str = "scry.core/description";
    pub const SUBTITLE: &str = "scry.core/subtitle";
    pub const PHOTO: &str = "scry.visual/photo";
    pub const ICON: &str = "scry.visual/icon";
    pub const AVATAR: &str = "scry.core/avatar";
    pub const URL: &str = "scry.core/url";
    pub const LINKS: &str = "scry.core/links";

    // --- Music ---
    pub const ISRC: &str = "scry.music/isrc";
    pub const GENRE: &str = "scry.music/genre";
    pub const DURATION: &str = "scry.music/duration_ms";

    // --- Place / Geo ---
    pub const LATITUDE: &str = "scry.geo/lat";
    pub const LONGITUDE: &str = "scry.geo/lon";
    pub const CITY: &str = "scry.core/city";
    pub const COUNTRY_CODE: &str = "scry.core/country_code";

    // --- Person ---
    pub const EMAIL: &str = "scry.person/email";
    pub const PHONE: &str = "scry.person/phone";
    pub const BIRTHDATE: &str = "scry.person/birthdate";

    // --- Status / Activity ---
    /// The entity currently being played or consumed by the user.
    pub const NOW_PLAYING: &str = "scry.status/now_playing";

    // --- Health ---
    pub const STEPS: &str = "scry.health/steps";
    pub const HEART_RATE: &str = "scry.health/heart_rate";
    pub const CALORIES: &str = "scry.health/calories";

    // --- Finance ---
    pub const AMOUNT: &str = "scry.finance/amount";
    pub const CURRENCY: &str = "scry.finance/currency";
}

pub mod predicates {
    // --- General ---
    /// Links an event to its primary actor (e.g. who sent the message).
    pub const SUBJECT: &str = "scry.core/subject";
    /// Links an event or entity to a location.
    pub const LOCATION: &str = "scry.core/location";
    /// General ownership or belonging.
    pub const OWNED_BY: &str = "scry.core/owned_by";

    // --- Music ---
    pub const PLAYED_BY: &str = "scry.music/played_by";
    pub const ON_ALBUM: &str = "scry.music/on_album";
    pub const IN_PLAYLIST: &str = "scry.music/in_playlist";

    // --- Media ---
    pub const AUTHOR: &str = "scry.media/author";
    pub const DIRECTOR: &str = "scry.media/director";
    pub const STARRING: &str = "scry.media/starring";

    // --- Social ---
    pub const FOLLOWS: &str = "scry.social/follows";
    pub const WORKS_FOR: &str = "scry.social/works_for";
    pub const MEMBER_OF: &str = "scry.social/member_of";
}
