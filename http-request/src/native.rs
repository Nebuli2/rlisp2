extern crate minihttp;
use minihttp::request::Request;

pub fn http_request(url: impl AsRef<str>) -> Option<String> {
    let mut req = Request::new(url.as_ref()).ok()?;
    let res = req.get().send().ok()?;
    Some(res.text())
}