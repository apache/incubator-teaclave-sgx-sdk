//! parsing server response
use crate::{
    error::{Error, ParseErr},
    uri::Uri,
};
use std::prelude::v1::*;
use std::{
    collections::{hash_map, HashMap},
    fmt,
    io::Write,
    str,
};
use unicase::Ascii;

pub(crate) const CR_LF_2: [u8; 4] = [13, 10, 13, 10];

///Represents an HTTP response.
///
///It contains `Headers` and `Status` parsed from response.
#[derive(Debug, PartialEq, Clone)]
pub struct Response {
    status: Status,
    headers: Headers,
}

impl Response {
    ///Creates new `Response` with head - status and headers - parsed from a slice of bytes
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const HEAD: &[u8; 102] = b"HTTP/1.1 200 OK\r\n\
    ///                         Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                         Content-Type: text/html\r\n\
    ///                         Content-Length: 100\r\n\r\n";
    ///
    ///let response = Response::from_head(HEAD).unwrap();
    ///```
    pub fn from_head(head: &[u8]) -> Result<Response, Error> {
        let mut head = str::from_utf8(head)?.splitn(2, '\n');

        let status = head.next().ok_or(ParseErr::StatusErr)?.parse()?;
        let headers = head.next().ok_or(ParseErr::HeadersErr)?.parse()?;

        Ok(Response { status, headers })
    }

    ///Parses `Response` from slice of bytes. Writes it's body to `writer`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///```
    pub fn try_from<T: Write>(res: &[u8], writer: &mut T) -> Result<Response, Error> {
        if res.is_empty() {
            Err(Error::Parse(ParseErr::Empty))
        } else {
            let mut pos = res.len();
            if let Some(v) = find_slice(res, &CR_LF_2) {
                pos = v;
            }

            let response = Self::from_head(&res[..pos])?;
            writer.write_all(&res[pos..])?;

            Ok(response)
        }
    }

    ///Returns status code of this `Response`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::{Response, StatusCode};
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///assert_eq!(response.status_code(), StatusCode::new(200));
    ///```
    pub fn status_code(&self) -> StatusCode {
        self.status.code
    }

    ///Returns HTTP version of this `Response`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///assert_eq!(response.version(), "HTTP/1.1");
    ///```
    pub fn version(&self) -> &str {
        &self.status.version
    }

    ///Returns reason of this `Response`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///assert_eq!(response.reason(), "OK");
    ///```
    pub fn reason(&self) -> &str {
        &self.status.reason
    }

    ///Returns headers of this `Response`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///let headers = response.headers();
    ///```
    pub fn headers(&self) -> &Headers {
        &self.headers
    }

    ///Returns length of the content of this `Response` as a `Result`, according to information
    ///included in headers. If there is no such an information, returns `Ok(0)`.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Response;
    ///
    ///const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
    ///                             Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
    ///                             Content-Type: text/html\r\n\
    ///                             Content-Length: 100\r\n\r\n\
    ///                             <html>hello\r\n\r\nhello</html>";
    ///let mut body = Vec::new();
    ///
    ///let response = Response::try_from(RESPONSE, &mut body).unwrap();
    ///assert_eq!(response.content_len().unwrap(), 100);
    ///```
    pub fn content_len(&self) -> Result<usize, ParseErr> {
        match self.headers().get("Content-Length") {
            Some(p) => Ok(p.parse()?),
            None => Ok(0),
        }
    }
}

///Status of HTTP response
#[derive(PartialEq, Debug, Clone)]
pub struct Status {
    version: String,
    code: StatusCode,
    reason: String,
}

impl<T, U, V> From<(T, U, V)> for Status
where
    T: ToString,
    V: ToString,
    StatusCode: From<U>,
{
    fn from(status: (T, U, V)) -> Status {
        Status {
            version: status.0.to_string(),
            code: StatusCode::from(status.1),
            reason: status.2.to_string(),
        }
    }
}

impl str::FromStr for Status {
    type Err = ParseErr;

    fn from_str(status_line: &str) -> Result<Status, Self::Err> {
        let mut status_line = status_line.trim().splitn(3, ' ');

        let version = status_line.next().ok_or(ParseErr::StatusErr)?;
        let code: StatusCode = status_line.next().ok_or(ParseErr::StatusErr)?.parse()?;
        let reason = status_line
            .next()
            .unwrap_or_else(|| code.reason().unwrap_or("Unknown"));

        Ok(Status::from((version, code, reason)))
    }
}

///Wrapper around HashMap<Ascii<String>, String> with additional functionality for parsing HTTP headers
///
///# Example
///```
///use http_req::response::Headers;
///
///let mut headers = Headers::new();
///headers.insert("Connection", "Close");
///
///assert_eq!(headers.get("Connection"), Some(&"Close".to_string()))
///```
#[derive(Debug, PartialEq, Clone, Default)]
pub struct Headers(HashMap<Ascii<String>, String>);

impl Headers {
    ///Creates an empty `Headers`.
    ///
    ///The headers are initially created with a capacity of 0, so they will not allocate until
    ///it is first inserted into.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Headers;
    ///
    ///let mut headers = Headers::new();
    ///```
    pub fn new() -> Headers {
        Headers(HashMap::new())
    }

    ///Creates empty `Headers` with the specified capacity.
    ///
    ///The headers will be able to hold at least capacity elements without reallocating.
    ///If capacity is 0, the headers will not allocate.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Headers;
    ///
    ///let mut headers = Headers::with_capacity(200);
    ///```
    pub fn with_capacity(capacity: usize) -> Headers {
        Headers(HashMap::with_capacity(capacity))
    }

    ///An iterator visiting all key-value pairs in arbitrary order.
    ///The iterator's element type is (&Ascii<String>, &String).
    ///
    ///# Examples
    ///```
    ///use http_req::response::Headers;
    ///
    ///let mut headers = Headers::new();
    ///headers.insert("Accept-Charset", "utf-8");
    ///headers.insert("Accept-Language", "en-US");
    ///headers.insert("Connection", "Close");
    ///
    ///let mut iterator = headers.iter();
    ///```
    pub fn iter(&self) -> hash_map::Iter<Ascii<String>, String> {
        self.0.iter()
    }

    ///Returns a reference to the value corresponding to the key.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Headers;
    ///
    ///let mut headers = Headers::new();
    ///headers.insert("Accept-Charset", "utf-8");
    ///
    ///assert_eq!(headers.get("Accept-Charset"), Some(&"utf-8".to_string()))
    ///```
    pub fn get<T: ToString + ?Sized>(&self, k: &T) -> Option<&std::string::String> {
        self.0.get(&Ascii::new(k.to_string()))
    }

    ///Inserts a key-value pair into the headers.
    ///
    ///If the headers did not have this key present, None is returned.
    ///
    ///If the headers did have this key present, the value is updated, and the old value is returned.
    ///The key is not updated, though; this matters for types that can be == without being identical.
    ///
    ///# Examples
    ///```
    ///use http_req::response::Headers;
    ///
    ///let mut headers = Headers::new();
    ///headers.insert("Accept-Language", "en-US");
    ///```
    pub fn insert<T, U>(&mut self, key: &T, val: &U) -> Option<String>
    where
        T: ToString + ?Sized,
        U: ToString + ?Sized,
    {
        self.0.insert(Ascii::new(key.to_string()), val.to_string())
    }

    ///Creates default headers for a HTTP request
    ///
    ///# Examples
    ///```
    ///use http_req::{response::Headers, uri::Uri};
    ///
    ///let uri: Uri = "https://www.rust-lang.org/learn".parse().unwrap();
    ///let headers = Headers::default_http(&uri);
    ///```
    pub fn default_http(uri: &Uri) -> Headers {
        let mut headers = Headers::with_capacity(4);

        headers.insert("Host", &uri.host_header().unwrap_or_default());
        headers.insert("Referer", &uri);

        headers
    }
}

impl str::FromStr for Headers {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<Headers, ParseErr> {
        let headers = s.trim();

        if headers.lines().all(|e| e.contains(':')) {
            let headers = headers
                .lines()
                .map(|elem| {
                    let idx = elem.find(':').unwrap();
                    let (key, value) = elem.split_at(idx);
                    (Ascii::new(key.to_string()), value[1..].trim().to_string())
                })
                .collect();

            Ok(Headers(headers))
        } else {
            Err(ParseErr::HeadersErr)
        }
    }
}

impl From<HashMap<Ascii<String>, String>> for Headers {
    fn from(map: HashMap<Ascii<String>, String>) -> Headers {
        Headers(map)
    }
}

impl From<Headers> for HashMap<Ascii<String>, String> {
    fn from(map: Headers) -> HashMap<Ascii<String>, String> {
        map.0
    }
}

impl fmt::Display for Headers {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let headers: String = self
            .iter()
            .map(|(key, val)| format!("  {}: {}\r\n", key, val))
            .collect();

        write!(f, "{{\r\n{}}}", headers)
    }
}

///Code sent by a server in response to a client's request.
///
///# Example
///```
///use http_req::response::StatusCode;
///
///const code: StatusCode = StatusCode::new(200);
///assert!(code.is_success())
///```
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct StatusCode(u16);

impl StatusCode {
    ///Creates new StatusCode from `u16` value.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(200);
    ///```
    pub const fn new(code: u16) -> StatusCode {
        StatusCode(code)
    }

    ///Checks if this `StatusCode` is within 100-199, which indicates that it's Informational.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(101);
    ///assert!(code.is_info())
    ///```
    pub fn is_info(self) -> bool {
        self.0 >= 100 && self.0 < 200
    }

    ///Checks if this `StatusCode` is within 200-299, which indicates that it's Successful.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(204);
    ///assert!(code.is_success())
    ///```
    pub fn is_success(self) -> bool {
        self.0 >= 200 && self.0 < 300
    }

    ///Checks if this `StatusCode` is within 300-399, which indicates that it's Redirection.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(301);
    ///assert!(code.is_redirect())
    ///```
    pub fn is_redirect(self) -> bool {
        self.0 >= 300 && self.0 < 400
    }

    ///Checks if this `StatusCode` is within 400-499, which indicates that it's Client Error.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(400);
    ///assert!(code.is_client_err())
    ///```
    pub fn is_client_err(self) -> bool {
        self.0 >= 400 && self.0 < 500
    }

    ///Checks if this `StatusCode` is within 500-599, which indicates that it's Server Error.
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(503);
    ///assert!(code.is_server_err())
    ///```
    pub fn is_server_err(self) -> bool {
        self.0 >= 500 && self.0 < 600
    }

    ///Checks this `StatusCode` using closure `f`
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(203);
    ///assert!(code.is(|i| i > 199 && i < 250))
    ///```
    pub fn is<F: FnOnce(u16) -> bool>(self, f: F) -> bool {
        f(self.0)
    }

    ///Returns `Reason-Phrase` corresponding to this `StatusCode`
    ///
    ///# Examples
    ///```
    ///use http_req::response::StatusCode;
    ///
    ///const code: StatusCode = StatusCode::new(200);
    ///assert_eq!(code.reason(), Some("OK"))
    ///```
    pub fn reason(self) -> Option<&'static str> {
        match self.0 {
            100 => Some("Continue"),
            101 => Some("Switching Protocols"),
            102 => Some("Processing"),
            200 => Some("OK"),
            201 => Some("Created"),
            202 => Some("Accepted"),
            203 => Some("Non Authoritative Information"),
            204 => Some("No Content"),
            205 => Some("Reset Content"),
            206 => Some("Partial Content"),
            207 => Some("Multi-Status"),
            208 => Some("Already Reported"),
            226 => Some("IM Used"),
            300 => Some("Multiple Choices"),
            301 => Some("Moved Permanently"),
            302 => Some("Found"),
            303 => Some("See Other"),
            304 => Some("Not Modified"),
            305 => Some("Use Proxy"),
            307 => Some("Temporary Redirect"),
            308 => Some("Permanent Redirect"),
            400 => Some("Bad Request"),
            401 => Some("Unauthorized"),
            402 => Some("Payment Required"),
            403 => Some("Forbidden"),
            404 => Some("Not Found"),
            405 => Some("Method Not Allowed"),
            406 => Some("Not Acceptable"),
            407 => Some("Proxy Authentication Required"),
            408 => Some("Request Timeout"),
            409 => Some("Conflict"),
            410 => Some("Gone"),
            411 => Some("Length Required"),
            412 => Some("Precondition Failed"),
            413 => Some("Payload Too Large"),
            414 => Some("URI Too Long"),
            415 => Some("Unsupported Media Type"),
            416 => Some("Range Not Satisfiable"),
            417 => Some("Expectation Failed"),
            418 => Some("I'm a teapot"),
            421 => Some("Misdirected Request"),
            422 => Some("Unprocessable Entity"),
            423 => Some("Locked"),
            424 => Some("Failed Dependency"),
            426 => Some("Upgrade Required"),
            428 => Some("Precondition Required"),
            429 => Some("Too Many Requests"),
            431 => Some("Request Header Fields Too Large"),
            451 => Some("Unavailable For Legal Reasons"),
            500 => Some("Internal Server Error"),
            501 => Some("Not Implemented"),
            502 => Some("Bad Gateway"),
            503 => Some("Service Unavailable"),
            504 => Some("Gateway Timeout"),
            505 => Some("HTTP Version Not Supported"),
            506 => Some("Variant Also Negotiates"),
            507 => Some("Insufficient Storage"),
            508 => Some("Loop Detected"),
            510 => Some("Not Extended"),
            511 => Some("Network Authentication Required"),
            _ => None,
        }
    }
}

impl From<StatusCode> for u16 {
    fn from(code: StatusCode) -> Self {
        code.0
    }
}

impl From<u16> for StatusCode {
    fn from(code: u16) -> Self {
        StatusCode(code)
    }
}

impl fmt::Display for StatusCode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl str::FromStr for StatusCode {
    type Err = ParseErr;

    fn from_str(s: &str) -> Result<StatusCode, ParseErr> {
        Ok(StatusCode::new(s.parse()?))
    }
}

///Finds elements slice `e` inside slice `data`. Returns position of the end of first match.
pub fn find_slice<T>(data: &[T], e: &[T]) -> Option<usize>
where
    [T]: PartialEq,
{
    for i in 0..=data.len() - e.len() {
        if data[i..(i + e.len())] == *e {
            return Some(i + e.len());
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const RESPONSE: &[u8; 129] = b"HTTP/1.1 200 OK\r\n\
                                         Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
                                         Content-Type: text/html\r\n\
                                         Content-Length: 100\r\n\r\n\
                                         <html>hello</html>\r\n\r\nhello";
    const RESPONSE_H: &[u8; 102] = b"HTTP/1.1 200 OK\r\n\
                                           Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
                                           Content-Type: text/html\r\n\
                                           Content-Length: 100\r\n\r\n";
    const BODY: &[u8; 27] = b"<html>hello</html>\r\n\r\nhello";

    const STATUS_LINE: &str = "HTTP/1.1 200 OK";
    const VERSION: &str = "HTTP/1.1";
    const CODE: u16 = 200;
    const REASON: &str = "OK";

    const HEADERS: &str = "Date: Sat, 11 Jan 2003 02:44:04 GMT\r\n\
                           Content-Type: text/html\r\n\
                           Content-Length: 100\r\n";
    const CODE_S: StatusCode = StatusCode(200);

    #[test]
    fn status_code_new() {
        assert_eq!(StatusCode::new(200), StatusCode(200));
        assert_ne!(StatusCode::new(400), StatusCode(404));
    }

    #[test]
    fn status_code_info() {
        for i in 100..200 {
            assert!(StatusCode::new(i).is_info())
        }

        for i in (0..1000).filter(|&i| i < 100 || i >= 200) {
            assert!(!StatusCode::new(i).is_info())
        }
    }

    #[test]
    fn status_code_success() {
        for i in 200..300 {
            assert!(StatusCode::new(i).is_success())
        }

        for i in (0..1000).filter(|&i| i < 200 || i >= 300) {
            assert!(!StatusCode::new(i).is_success())
        }
    }

    #[test]
    fn status_code_redirect() {
        for i in 300..400 {
            assert!(StatusCode::new(i).is_redirect())
        }

        for i in (0..1000).filter(|&i| i < 300 || i >= 400) {
            assert!(!StatusCode::new(i).is_redirect())
        }
    }

    #[test]
    fn status_code_client_err() {
        for i in 400..500 {
            assert!(StatusCode::new(i).is_client_err())
        }

        for i in (0..1000).filter(|&i| i < 400 || i >= 500) {
            assert!(!StatusCode::new(i).is_client_err())
        }
    }

    #[test]
    fn status_code_server_err() {
        for i in 500..600 {
            assert!(StatusCode::new(i).is_server_err())
        }

        for i in (0..1000).filter(|&i| i < 500 || i >= 600) {
            assert!(!StatusCode::new(i).is_server_err())
        }
    }

    #[test]
    fn status_code_is() {
        let check = |i| i % 3 == 0;

        let code_1 = StatusCode::new(200);
        let code_2 = StatusCode::new(300);

        assert!(!code_1.is(check));
        assert!(code_2.is(check));
    }

    #[test]
    fn status_code_reason() {
        assert_eq!(StatusCode(200).reason(), Some("OK"));
        assert_eq!(StatusCode(400).reason(), Some("Bad Request"));
    }

    #[test]
    fn status_code_from() {
        assert_eq!(StatusCode::from(200), StatusCode(200));
    }

    #[test]
    fn u16_from_status_code() {
        assert_eq!(u16::from(CODE_S), 200);
    }

    #[test]
    fn status_code_display() {
        let code = format!("Status Code: {}", StatusCode::new(200));
        const CODE_EXPECT: &str = "Status Code: 200";

        assert_eq!(code, CODE_EXPECT);
    }

    #[test]
    fn status_code_from_str() {
        assert_eq!("200".parse::<StatusCode>(), Ok(StatusCode(200)));
        assert_ne!("400".parse::<StatusCode>(), Ok(StatusCode(404)));
    }

    #[test]
    fn status_from() {
        let status = Status::from((VERSION, CODE, REASON));

        assert_eq!(status.version, VERSION);
        assert_eq!(status.code, CODE_S);
        assert_eq!(status.reason, REASON);
    }

    #[test]
    fn status_from_str() {
        let status = STATUS_LINE.parse::<Status>().unwrap();

        assert_eq!(status.version, VERSION);
        assert_eq!(status.code, CODE_S);
        assert_eq!(status.reason, REASON);
    }

    #[test]
    fn headers_new() {
        assert_eq!(Headers::new(), Headers(HashMap::new()));
    }

    #[test]
    fn headers_get() {
        let mut headers = Headers::with_capacity(2);
        headers.insert("Date", "Sat, 11 Jan 2003 02:44:04 GMT");

        assert_eq!(
            headers.get("Date"),
            Some(&"Sat, 11 Jan 2003 02:44:04 GMT".to_string())
        );
    }

    #[test]
    fn headers_insert() {
        let mut headers_expect = HashMap::new();
        headers_expect.insert(Ascii::new("Connection".to_string()), "Close".to_string());
        let headers_expect = Headers(headers_expect);

        let mut headers = Headers::new();
        headers.insert("Connection", "Close");

        assert_eq!(headers_expect, headers);
    }

    #[test]
    fn headers_default_http() {
        let uri = "http://doc.rust-lang.org/std/string/index.html"
            .parse()
            .unwrap();

        let mut headers = Headers::with_capacity(4);
        headers.insert("Host", "doc.rust-lang.org");
        headers.insert("Referer", "http://doc.rust-lang.org/std/string/index.html");

        assert_eq!(Headers::default_http(&uri), headers);
    }

    #[test]
    fn headers_from_str() {
        let mut headers_expect = HashMap::with_capacity(2);
        headers_expect.insert(
            Ascii::new("Date".to_string()),
            "Sat, 11 Jan 2003 02:44:04 GMT".to_string(),
        );
        headers_expect.insert(
            Ascii::new("Content-Type".to_string()),
            "text/html".to_string(),
        );
        headers_expect.insert(Ascii::new("Content-Length".to_string()), "100".to_string());

        let headers = HEADERS.parse::<Headers>().unwrap();
        assert_eq!(headers, Headers::from(headers_expect));
    }

    #[test]
    fn headers_from() {
        let mut headers_expect = HashMap::with_capacity(4);
        headers_expect.insert(
            Ascii::new("Date".to_string()),
            "Sat, 11 Jan 2003 02:44:04 GMT".to_string(),
        );
        headers_expect.insert(
            Ascii::new("Content-Type".to_string()),
            "text/html".to_string(),
        );
        headers_expect.insert(Ascii::new("Content-Length".to_string()), "100".to_string());

        assert_eq!(
            Headers(headers_expect.clone()),
            Headers::from(headers_expect)
        );
    }

    #[test]
    fn headers_case_insensitive() {
        let header_names = ["Host", "host", "HOST", "HoSt"];
        let mut headers = Headers::with_capacity(1);
        headers.insert("Host", "doc.rust-lang.org");

        for name in header_names.iter() {
            assert_eq!(headers.get(name), Some(&"doc.rust-lang.org".to_string()));
        }
    }

    #[test]
    fn hash_map_from_headers() {
        let mut headers = Headers::with_capacity(4);
        headers.insert("Date", "Sat, 11 Jan 2003 02:44:04 GMT");
        headers.insert("Content-Type", "text/html");
        headers.insert("Content-Length", "100");

        let mut headers_expect = HashMap::with_capacity(4);
        headers_expect.insert(
            Ascii::new("Date".to_string()),
            "Sat, 11 Jan 2003 02:44:04 GMT".to_string(),
        );
        headers_expect.insert(
            Ascii::new("Content-Type".to_string()),
            "text/html".to_string(),
        );
        headers_expect.insert(Ascii::new("Content-Length".to_string()), "100".to_string());

        assert_eq!(HashMap::from(headers), headers_expect);
    }

    #[test]
    fn find_slice_e() {
        const WORDS: [&str; 8] = ["Good", "job", "Great", "work", "Have", "fun", "See", "you"];
        const SEARCH: [&str; 3] = ["Great", "work", "Have"];

        assert_eq!(find_slice(&WORDS, &SEARCH), Some(5));
    }

    #[test]
    fn res_from_head() {
        Response::from_head(RESPONSE_H).unwrap();
    }

    #[test]
    fn res_try_from() {
        let mut writer = Vec::new();

        Response::try_from(RESPONSE, &mut writer).unwrap();
        Response::try_from(RESPONSE_H, &mut writer).unwrap();
    }

    #[test]
    #[should_panic]
    fn res_from_empty() {
        let mut writer = Vec::new();
        Response::try_from(&[], &mut writer).unwrap();
    }

    #[test]
    fn res_status_code() {
        let mut writer = Vec::new();
        let res = Response::try_from(RESPONSE, &mut writer).unwrap();

        assert_eq!(res.status_code(), CODE_S);
    }

    #[test]
    fn res_version() {
        let mut writer = Vec::new();
        let res = Response::try_from(RESPONSE, &mut writer).unwrap();

        assert_eq!(res.version(), "HTTP/1.1");
    }

    #[test]
    fn res_reason() {
        let mut writer = Vec::new();
        let res = Response::try_from(RESPONSE, &mut writer).unwrap();

        assert_eq!(res.reason(), "OK");
    }

    #[test]
    fn res_headers() {
        let mut writer = Vec::new();
        let res = Response::try_from(RESPONSE, &mut writer).unwrap();

        let mut headers = Headers::with_capacity(2);
        headers.insert("Date", "Sat, 11 Jan 2003 02:44:04 GMT");
        headers.insert("Content-Type", "text/html");
        headers.insert("Content-Length", "100");

        assert_eq!(res.headers(), &Headers::from(headers));
    }

    #[test]
    fn res_content_len() {
        let mut writer = Vec::with_capacity(101);
        let res = Response::try_from(RESPONSE, &mut writer).unwrap();

        assert_eq!(res.content_len(), Ok(100));
    }

    #[test]
    fn res_body() {
        let mut writer = Vec::new();
        Response::try_from(RESPONSE, &mut writer).unwrap();

        assert_eq!(writer, BODY);
    }
}
