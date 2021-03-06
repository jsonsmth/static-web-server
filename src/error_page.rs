use iron::mime;
use iron::prelude::*;
use iron::status;
use iron::AfterMiddleware;
use std::fs;
use std::path::Path;

const PAGE_404: &str = "<h2>404</h2><p>Content could not found</p>";
const PAGE_50X: &str =
    "<h2>50x</h2><p>SERVICE is temporarily unavailable due an unexpected error</p>";

/// Custom Error pages middleware for Iron
pub struct ErrorPage {
    /// HTML file content for 404 errors.
    pub page404: String,
    /// HTML file content for 50x errors.
    pub page50x: String,
}

impl ErrorPage {
    /// Create a new instance of `ErrorPage` middleware with a given html pages.
    pub fn new<P: AsRef<Path>>(page_404_path: P, page_50x_path: P) -> ErrorPage {
        let page404 = if Path::new(&page_404_path.as_ref()).exists() {
            fs::read_to_string(page_404_path).unwrap()
        } else {
            String::from(PAGE_404)
        };

        let page50x = if Path::new(&page_50x_path.as_ref()).exists() {
            fs::read_to_string(page_50x_path).unwrap()
        } else {
            String::from(PAGE_50X)
        };

        ErrorPage { page404, page50x }
    }
}

impl AfterMiddleware for ErrorPage {
    fn after(&self, req: &mut Request, resp: Response) -> IronResult<Response> {
        let mut no_status_error = false;

        let content_type = "text/html"
            .parse::<mime::Mime>()
            .expect("Unable to create a default content type header");

        let mut resp = match resp.status {
            Some(status::NotFound) => {
                Response::with((content_type, status::NotFound, self.page404.as_str()))
            }
            Some(status::InternalServerError) => Response::with((
                content_type,
                status::InternalServerError,
                self.page50x.as_str(),
            )),
            Some(status::BadGateway) => {
                Response::with((content_type, status::BadGateway, self.page50x.as_str()))
            }
            Some(status::ServiceUnavailable) => Response::with((
                content_type,
                status::ServiceUnavailable,
                self.page50x.as_str(),
            )),
            Some(status::GatewayTimeout) => {
                Response::with((content_type, status::GatewayTimeout, self.page50x.as_str()))
            }
            _ => {
                no_status_error = true;
                resp
            }
        };

        // Empty response body only on HEAD requests and status error (404,50x)
        if req.method == iron::method::Head && !no_status_error {
            resp.set_mut(vec![]);
        }

        Ok(resp)
    }
}
