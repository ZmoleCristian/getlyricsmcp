use crate::model::Site;
use crate::sites::slug;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slug {
    Hyphen,
    Compact,
    CompactNoThe,
    Underscore,
    TurkishHyphen,
}

impl Slug {
    fn apply(self, input: &str) -> String {
        match self {
            Slug::Hyphen => slug::hyphenated(input),
            Slug::Compact => slug::compact(input),
            Slug::CompactNoThe => slug::compact(slug::strip_leading_the(input)),
            Slug::Underscore => slug::hyphenated(input).replace('-', "_"),
            Slug::TurkishHyphen => slug::hyphenated(&slug::fold_turkish(input)),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Extract {
    First,
    All,
    SiblingDiv,
    BareDivs,
    WalkSkipNoise,
}

#[derive(Debug, Clone, Copy)]
pub struct SiteSpec {
    pub site: Site,
    pub template: &'static str,
    pub artist_slug: Slug,
    pub title_slug: Slug,
    pub selector: &'static str,
    pub extract: Extract,
    pub verify_title: bool,
}

impl SiteSpec {
    pub fn url(&self, artist: &str, title: &str) -> String {
        self.template
            .replace("{artist}", &self.artist_slug.apply(artist))
            .replace("{title}", &self.title_slug.apply(title))
    }
}

pub const SPECS: [SiteSpec; 12] = [
    SiteSpec {
        site: Site::AzLyrics,
        template: "https://www.azlyrics.com/lyrics/{artist}/{title}.html",
        artist_slug: Slug::CompactNoThe,
        title_slug: Slug::Compact,
        selector: "div.ringtone",
        extract: Extract::SiblingDiv,
        verify_title: false,
    },
    SiteSpec {
        site: Site::Genius,
        template: "https://genius.com/{artist}-{title}-lyrics",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "[data-lyrics-container=\"true\"]",
        extract: Extract::All,
        verify_title: false,
    },
    SiteSpec {
        site: Site::VersuriRo,
        template: "https://www.versuri.ro/versuri/{artist}-{title}/",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "#textdiv",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::VersuriUs,
        template: "https://www.versuri.us/{artist}-{title}-lyrics",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "div.post_inside p",
        extract: Extract::All,
        verify_title: false,
    },
    SiteSpec {
        site: Site::Lyricshare,
        template: "https://lyricshare.net/ru/{artist}/{title}.html",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "#lyricSheet",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::Tekstowo,
        template: "https://www.tekstowo.pl/{artist}/{title}",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "#songText > .inner-text",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::LetrasMusBr,
        template: "https://www.letras.mus.br/{artist}/{title}/",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "div.lyric-original",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::Sarkisozum,
        template: "https://www.sarkisozum.gen.tr/en/{artist}/{title}-lyrics",
        artist_slug: Slug::TurkishHyphen,
        title_slug: Slug::TurkishHyphen,
        selector: "#contentArea div.toolbox + div",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::Klyrics,
        template: "https://klyrics.net/{artist}-{title}/",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "div.hangul-text",
        extract: Extract::First,
        verify_title: false,
    },
    SiteSpec {
        site: Site::ParolesNet,
        template: "https://www.paroles.net/{artist}/paroles-{title}",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "article.lyrics",
        extract: Extract::BareDivs,
        verify_title: true,
    },
    SiteSpec {
        site: Site::Angolotesti,
        template: "https://angolotesti.it/a/testi_canzoni_{artist}/testo_canzone_{title}.html",
        artist_slug: Slug::Underscore,
        title_slug: Slug::Underscore,
        selector: "div.testo",
        extract: Extract::WalkSkipNoise,
        verify_title: true,
    },
    SiteSpec {
        site: Site::LetrasCom,
        template: "https://www.letras.com/{artist}/{title}/",
        artist_slug: Slug::Hyphen,
        title_slug: Slug::Hyphen,
        selector: "div.lyric-original",
        extract: Extract::First,
        verify_title: true,
    },
];
