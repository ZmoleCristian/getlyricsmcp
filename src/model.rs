use serde::{Serialize, Serializer};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Site {
    AzLyrics,
    Genius,
    SoundCloud,
    VersuriRo,
    Lyricshare,
    ParolesNet,
    Tekstowo,
    LetrasMusBr,
    Angolotesti,
    LetrasCom,
    Sarkisozum,
    Klyrics,
    VersuriUs,
}

impl Serialize for Site {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.label())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reliability {
    Low,
    Medium,
    High,
}

impl Site {
    pub fn label(self) -> &'static str {
        match self {
            Site::AzLyrics => "azlyrics",
            Site::Genius => "genius",
            Site::SoundCloud => "soundcloud",
            Site::VersuriRo => "versuri.ro",
            Site::Lyricshare => "lyricshare.net",
            Site::ParolesNet => "paroles.net",
            Site::Tekstowo => "tekstowo.pl",
            Site::LetrasMusBr => "letras.mus.br",
            Site::Angolotesti => "angolotesti.it",
            Site::LetrasCom => "letras.com",
            Site::Sarkisozum => "sarkisozum.gen.tr",
            Site::Klyrics => "klyrics.net",
            Site::VersuriUs => "versuri.us",
        }
    }

    pub fn reliability(self) -> Reliability {
        match self {
            Site::AzLyrics
            | Site::Genius
            | Site::VersuriRo
            | Site::Lyricshare
            | Site::Tekstowo
            | Site::LetrasMusBr
            | Site::Sarkisozum
            | Site::Klyrics
            | Site::VersuriUs => Reliability::High,
            Site::ParolesNet | Site::Angolotesti | Site::LetrasCom => Reliability::Medium,
            Site::SoundCloud => Reliability::Low,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct SearchHit {
    pub id: String,
    pub site: Site,
    pub url: String,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct Lyrics {
    pub site: Site,
    pub url: String,
    pub text: String,
}
