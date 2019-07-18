//! uri operations
use crate::error::{Error, ParseErr};
use std::prelude::v1::*;
use std::{
    fmt,
    ops::{Index, Range},
    str,
};

const HTTP_PORT: u16 = 80;
const HTTPS_PORT: u16 = 443;

///A (half-open) range bounded inclusively below and exclusively above (start..end) with `Copy`.
#[derive(Copy, Clone, Debug, PartialOrd, PartialEq)]
pub struct RangeC {
    pub start: usize,
    pub end: usize,
}

impl RangeC {
    ///Creates new `RangeC` with `start` and `end`.
    ///
    ///# Exmaples
    ///```
    ///use http_req::uri::RangeC;
    ///
    ///const range: RangeC = RangeC::new(0, 20);
    ///```
    pub const fn new(start: usize, end: usize) -> RangeC {
        RangeC { start, end }
    }
}

impl From<RangeC> for Range<usize> {
    fn from(range: RangeC) -> Range<usize> {
        Range {
            start: range.start,
            end: range.end,
        }
    }
}

impl Index<RangeC> for String {
    type Output = str;

    #[inline]
    fn index(&self, index: RangeC) -> &str {
        &self[..][Range::from(index)]
    }
}

///Representation of Uniform Resource Identifier
///
///# Example
///```
///use http_req::uri::Uri;
///
///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
///assert_eq!(uri.host(), Some("foo.com"));
///```
#[derive(Clone, Debug, PartialEq)]
pub struct Uri {
    inner: String,
    scheme: RangeC,
    authority: Option<Authority>,
    path: Option<RangeC>,
    query: Option<RangeC>,
    fragment: Option<RangeC>,
}

impl Uri {
    ///Returns scheme of this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.scheme(), "https");
    ///```
    pub fn scheme(&self) -> &str {
        &self.inner[self.scheme]
    }

    ///Returns information about the user included in this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.user_info(), Some("user:info"));
    ///```
    pub fn user_info(&self) -> Option<&str> {
        match self.authority {
            Some(ref a) => a.user_info(),
            None => None,
        }
    }

    ///Returns host of this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.host(), Some("foo.com"));
    ///```
    pub fn host(&self) -> Option<&str> {
        match self.authority {
            Some(ref a) => Some(a.host()),
            None => None,
        }
    }

    ///Returns host of this `Uri` to use in a header.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.host_header(), Some("foo.com:12".to_string()));
    ///```
    pub fn host_header(&self) -> Option<String> {
        match self.host() {
            Some(h) => match self.corr_port() {
                HTTP_PORT | HTTPS_PORT => Some(h.to_string()),
                p => Some(format!("{}:{}", h, p)),
            },
            _ => None,
        }
    }

    ///Returns port of this `Uri`
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.port(), Some(12));
    ///```
    pub fn port(&self) -> Option<u16> {
        match &self.authority {
            Some(a) => a.port(),
            None => None,
        }
    }

    ///Returns port corresponding to this `Uri`.
    ///Returns default port if it hasn't been set in the uri.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.corr_port(), 12);
    ///```
    pub fn corr_port(&self) -> u16 {
        let default_port = match self.scheme() {
            "https" => HTTPS_PORT,
            _ => HTTP_PORT,
        };

        match self.authority {
            Some(ref a) => a.port().unwrap_or(default_port),
            None => default_port,
        }
    }

    ///Returns path of this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.path(), Some("/bar/baz"));
    ///```
    pub fn path(&self) -> Option<&str> {
        self.path.map(|r| &self.inner[r])
    }

    ///Returns query of this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.query(), Some("query"));
    ///```
    pub fn query(&self) -> Option<&str> {
        self.query.map(|r| &self.inner[r])
    }

    ///Returns fragment of this `Uri`.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.fragment(), Some("fragment"));
    ///```
    pub fn fragment(&self) -> Option<&str> {
        self.fragment.map(|r| &self.inner[r])
    }

    ///Returns resource `Uri` points to.
    ///
    ///# Example
    ///```
    ///use http_req::uri::Uri;
    ///
    ///let uri: Uri = "https://user:info@foo.com:12/bar/baz?query#fragment".parse().unwrap();
    ///assert_eq!(uri.resource(), "/bar/baz?query#fragment");
    ///```
    pub fn resource(&self) -> &str {
        let mut result = "/";

        for v in &[self.path, self.query, self.fragment] {
            if let Some(r) = v {
                result = &self.inner[r.start..];
                break;
            }
        }

        result
    }
}

impl fmt::Display for Uri {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let uri = if let Some(auth) = &self.authority {
            let mut uri = self.inner.to_string();
            let auth = auth.to_string();
            let start = self.scheme.end + 3;

            uri.replace_range(start..(start + auth.len()), &auth);
            uri
        } else {
            self.inner.to_string()
        };

        write!(f, "{}", uri)
    }
}

impl str::FromStr for Uri {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s.to_string();
        remove_spaces(&mut s);

        let (scheme, mut uri_part) = get_chunks(&s, Some(RangeC::new(0, s.len())), ":");
        let scheme = scheme.ok_or(ParseErr::UriErr)?;

        let mut authority = None;

        if let Some(u) = &uri_part {
            if s[*u].contains("//") {
                let (auth, part) = get_chunks(&s, Some(RangeC::new(u.start + 2, u.end)), "/");

                authority = if let Some(a) = auth {
                    Some(s[a].parse()?)
                } else {
                    None
                };

                uri_part = part;
            }
        }

        let (mut path, uri_part) = get_chunks(&s, uri_part, "?");

        if authority.is_some() || &s[scheme] == "file" {
            path = path.map(|p| RangeC::new(p.start - 1, p.end));
        }

        let (query, fragment) = get_chunks(&s, uri_part, "#");

        Ok(Uri {
            inner: s,
            scheme,
            authority,
            path,
            query,
            fragment,
        })
    }
}

///Authority of Uri
///
///# Example
///```
///use http_req::uri::Authority;
///
///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
///assert_eq!(auth.host(), "foo.com");
///```
#[derive(Clone, Debug, PartialEq)]
pub struct Authority {
    inner: String,
    username: Option<RangeC>,
    password: Option<RangeC>,
    host: RangeC,
    port: Option<RangeC>,
}

impl Authority {
    ///Returns username of this `Authority`
    ///
    ///# Example
    ///```
    ///use http_req::uri::Authority;
    ///
    ///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
    ///assert_eq!(auth.username(), Some("user"));
    ///```
    pub fn username(&self) -> Option<&str> {
        self.username.map(|r| &self.inner[r])
    }

    ///Returns password of this `Authority`
    ///
    ///# Example
    ///```
    ///use http_req::uri::Authority;
    ///
    ///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
    ///assert_eq!(auth.password(), Some("info"));
    ///```
    pub fn password(&self) -> Option<&str> {
        self.password.map(|r| &self.inner[r])
    }

    ///Returns information about the user
    ///
    ///# Example
    ///```
    ///use http_req::uri::Authority;
    ///
    ///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
    ///assert_eq!(auth.user_info(), Some("user:info"));
    ///```
    pub fn user_info(&self) -> Option<&str> {
        match (&self.username, &self.password) {
            (Some(u), Some(p)) => Some(&self.inner[u.start..p.end]),
            (Some(u), None) => Some(&self.inner[*u]),
            _ => None,
        }
    }

    ///Returns host of this `Authority`
    ///
    ///# Example
    ///```
    ///use http_req::uri::Authority;
    ///
    ///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
    ///assert_eq!(auth.host(), "foo.com");
    ///```
    pub fn host(&self) -> &str {
        &self.inner[self.host]
    }

    ///Returns port of this `Authority`
    ///
    ///# Example
    ///```
    ///use http_req::uri::Authority;
    ///
    ///let auth: Authority = "user:info@foo.com:443".parse().unwrap();
    ///assert_eq!(auth.port(), Some(443));
    ///```
    pub fn port(&self) -> Option<u16> {
        match &self.port {
            Some(p) => Some(self.inner[*p].parse().unwrap()),
            None => None,
        }
    }
}

impl str::FromStr for Authority {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inner = s.to_string();

        let mut username = None;
        let mut password = None;

        let uri_part = if s.contains('@') {
            let (info, part) = get_chunks(&s, Some(RangeC::new(0, s.len())), "@");
            let (name, pass) = get_chunks(&s, info, ":");

            username = name;
            password = pass;

            part
        } else {
            Some(RangeC::new(0, s.len()))
        };

        let (host, port) = get_chunks(&s, uri_part, ":");
        let host = host.ok_or(ParseErr::UriErr)?;

        if let Some(p) = port {
            if inner[p].parse::<u16>().is_err() {
                return Err(ParseErr::UriErr);
            }
        }

        Ok(Authority {
            inner,
            username,
            password,
            host,
            port,
        })
    }
}

impl fmt::Display for Authority {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let auth = if let Some(pass) = self.password {
            let range = Range::from(pass);

            let hidden_pass = "*".repeat(range.len());
            let mut auth = self.inner.to_string();
            auth.replace_range(range, &hidden_pass);

            auth
        } else {
            self.inner.to_string()
        };

        write!(f, "{}", auth)
    }
}

//Removes whitespace from `text`
fn remove_spaces(text: &mut String) {
    text.retain(|c| !c.is_whitespace());
}

//Splits `s` by `separator`. If `separator` is found inside `s`, it will return two `Some` values
//consisting `RangeC` of each `&str`. If `separator` is at the end of `s` or it's not found,
//it will return tuple consisting `Some` with `RangeC` of entire `s` inside and None.
fn get_chunks<'a>(
    s: &'a str,
    range: Option<RangeC>,
    separator: &'a str,
) -> (Option<RangeC>, Option<RangeC>) {
    if let Some(r) = range {
        let range = Range::from(r);

        match s[range.clone()].find(separator) {
            Some(i) => {
                let before = Some(RangeC::new(r.start, r.start + i)).filter(|r| r.start != r.end);
                let after = Some(RangeC::new(r.start + i + 1, r.end)).filter(|r| r.start != r.end);

                (before, after)
            }
            None => {
                if !s[range].is_empty() {
                    (Some(r), None)
                } else {
                    (None, None)
                }
            }
        }
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_URIS: [&str; 4] = [
        "https://user:info@foo.com:12/bar/baz?query#fragment",
        "file:///C:/Users/User/Pictures/screenshot.png",
        "https://en.wikipedia.org/wiki/Hypertext_Transfer_Protocol",
        "mailto:John.Doe@example.com",
    ];

    const TEST_AUTH: [&str; 3] = [
        "user:info@foo.com:12",
        "en.wikipedia.org",
        "John.Doe@example.com",
    ];

    #[test]
    fn remove_space() {
        let mut text = String::from("Hello World     !");
        let expect = String::from("HelloWorld!");

        remove_spaces(&mut text);
        assert_eq!(text, expect);
    }

    #[test]
    fn uri_full_parse() {
        let uri = "abc://username:password@example.com:123/path/data?key=value&key2=value2#fragid1"
            .parse::<Uri>()
            .unwrap();
        assert_eq!(uri.scheme(), "abc");

        assert_eq!(uri.user_info(), Some("username:password"));
        assert_eq!(uri.host(), Some("example.com"));
        assert_eq!(uri.port(), Some(123));

        assert_eq!(uri.path(), Some("/path/data"));
        assert_eq!(uri.query(), Some("key=value&key2=value2"));
        assert_eq!(uri.fragment(), Some("fragid1"));
    }

    #[test]
    fn uri_parse() {
        for uri in TEST_URIS.iter() {
            uri.parse::<Uri>().unwrap();
        }
    }

    #[test]
    fn uri_scheme() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].scheme(), "https");
        assert_eq!(uris[1].scheme(), "file");
        assert_eq!(uris[2].scheme(), "https");
        assert_eq!(uris[3].scheme(), "mailto");
    }

    #[test]
    fn uri_uesr_info() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].user_info(), Some("user:info"));
        assert_eq!(uris[1].user_info(), None);
        assert_eq!(uris[2].user_info(), None);
        assert_eq!(uris[3].user_info(), None);
    }

    #[test]
    fn uri_host() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].host(), Some("foo.com"));
        assert_eq!(uris[1].host(), None);
        assert_eq!(uris[2].host(), Some("en.wikipedia.org"));
        assert_eq!(uris[3].host(), None);
    }

    #[test]
    fn uri_host_header() {
        let uri_def: Uri = "https://en.wikipedia.org:443/wiki/Hypertext_Transfer_Protocol"
            .parse()
            .unwrap();
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].host_header(), Some("foo.com:12".to_string()));
        assert_eq!(uris[2].host_header(), Some("en.wikipedia.org".to_string()));
        assert_eq!(uri_def.host_header(), Some("en.wikipedia.org".to_string()));
    }

    #[test]
    fn uri_port() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].port(), Some(12));

        for uri in uris.iter().skip(1) {
            assert_eq!(uri.port(), None);
        }
    }

    #[test]
    fn uri_corr_port() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].corr_port(), 12);
        assert_eq!(uris[1].corr_port(), HTTP_PORT);
        assert_eq!(uris[2].corr_port(), HTTPS_PORT);
        assert_eq!(uris[3].corr_port(), HTTP_PORT);
    }

    #[test]
    fn uri_path() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].path(), Some("/bar/baz"));
        assert_eq!(
            uris[1].path(),
            Some("/C:/Users/User/Pictures/screenshot.png")
        );
        assert_eq!(uris[2].path(), Some("/wiki/Hypertext_Transfer_Protocol"));
        assert_eq!(uris[3].path(), Some("John.Doe@example.com"));
    }

    #[test]
    fn uri_query() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].query(), Some("query"));

        for i in 1..3 {
            assert_eq!(uris[i].query(), None);
        }
    }

    #[test]
    fn uri_fragment() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].fragment(), Some("fragment"));

        for i in 1..3 {
            assert_eq!(uris[i].fragment(), None);
        }
    }

    #[test]
    fn uri_resource() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(uris[0].resource(), "/bar/baz?query#fragment");
        assert_eq!(uris[1].resource(), "/C:/Users/User/Pictures/screenshot.png");
        assert_eq!(uris[2].resource(), "/wiki/Hypertext_Transfer_Protocol");
        assert_eq!(uris[3].resource(), "John.Doe@example.com");
    }

    #[test]
    fn uri_display() {
        let uris: Vec<_> = TEST_URIS
            .iter()
            .map(|uri| uri.parse::<Uri>().unwrap())
            .collect();

        assert_eq!(
            uris[0].to_string(),
            "https://user:****@foo.com:12/bar/baz?query#fragment"
        );

        for i in 1..uris.len() {
            let s = uris[i].to_string();
            assert_eq!(s, TEST_URIS[i]);
        }
    }

    #[test]
    fn authority_username() {
        let auths: Vec<_> = TEST_AUTH
            .iter()
            .map(|auth| auth.parse::<Authority>().unwrap())
            .collect();

        assert_eq!(auths[0].username(), Some("user"));
        assert_eq!(auths[1].username(), None);
        assert_eq!(auths[2].username(), Some("John.Doe"));
    }

    #[test]
    fn authority_password() {
        let auths: Vec<_> = TEST_AUTH
            .iter()
            .map(|auth| auth.parse::<Authority>().unwrap())
            .collect();

        assert_eq!(auths[0].password(), Some("info"));
        assert_eq!(auths[1].password(), None);
        assert_eq!(auths[2].password(), None);
    }

    #[test]
    fn authority_host() {
        let auths: Vec<_> = TEST_AUTH
            .iter()
            .map(|auth| auth.parse::<Authority>().unwrap())
            .collect();

        assert_eq!(auths[0].host(), "foo.com");
        assert_eq!(auths[1].host(), "en.wikipedia.org");
        assert_eq!(auths[2].host(), "example.com");
    }

    #[test]
    fn authority_port() {
        let auths: Vec<_> = TEST_AUTH
            .iter()
            .map(|auth| auth.parse::<Authority>().unwrap())
            .collect();

        assert_eq!(auths[0].port(), Some(12));
        assert_eq!(auths[1].port(), None);
        assert_eq!(auths[2].port(), None);
    }

    #[test]
    fn authority_from_str() {
        for auth in TEST_AUTH.iter() {
            auth.parse::<Authority>().unwrap();
        }
    }

    #[test]
    fn authority_display() {
        let auths: Vec<_> = TEST_AUTH
            .iter()
            .map(|auth| auth.parse::<Authority>().unwrap())
            .collect();

        assert_eq!("user:****@foo.com:12", auths[0].to_string());

        for i in 1..auths.len() {
            let s = auths[i].to_string();
            assert_eq!(s, TEST_AUTH[i]);
        }
    }

    #[test]
    fn range_c_new() {
        assert_eq!(
            RangeC::new(22, 343),
            RangeC {
                start: 22,
                end: 343,
            }
        )
    }

    #[test]
    fn range_from_range_c() {
        assert_eq!(
            Range::from(RangeC::new(222, 43)),
            Range {
                start: 222,
                end: 43,
            }
        )
    }

    #[test]
    fn range_c_index() {
        const RANGE: RangeC = RangeC::new(0, 4);
        let text = TEST_AUTH[0].to_string();

        assert_eq!(text[..4], text[RANGE])
    }
}
