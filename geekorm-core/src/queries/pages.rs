//! # GeekORM Pages

/// Default limit for max page size
const DEFAULT_LIMIT: u32 = 100;

/// Page struct for pagination.
///
/// This is a simple struct to handle pagination for queries.
///
/// ```rust
/// # use geekorm::prelude::*;
///
/// #[derive(Table, Debug, Default, Clone, serde::Serialize, serde::Deserialize)]
/// pub struct Users {
///     pub id: PrimaryKeyInteger,
///     pub username: String,
///     pub age: i32,
///     pub postcode: Option<String>,
/// }
///
/// # fn main() {
/// // Create a new Page instance
/// let mut page = Page::new();
/// # assert_eq!(page.limit(), 100);
/// # assert_eq!(page.offset(), 0);
///
/// // Update the page to the next page
/// page.next();
/// # assert_eq!(page.page(), 1);
/// # assert_eq!(page.limit(), 100);
/// # assert_eq!(page.offset(), 100);
///
/// # page.next();
/// # assert_eq!(page.offset(), 200);
/// # page.prev();
///
/// // Build a query to select rows from the table
/// let select_query = Users::query_select()
///     .where_eq("username", "geekmasher")
///     .page(&page)
///     .order_by("age", QueryOrder::Asc)
///     .build()
///     .expect("Failed to build select query");
/// # assert_eq!(
/// #     select_query.query,
/// #     "SELECT id, username, age, postcode FROM Users WHERE username = ? ORDER BY age ASC LIMIT 100 OFFSET 100;"
/// # );
///
/// let page_max = Page::from((1, 10_000));
/// # assert_eq!(page_max.limit(), 100);
///
/// let option_page = Page::from((Some(5), Some(10)));
/// # assert_eq!(option_page.page(), 5);
/// # assert_eq!(option_page.limit(), 10);
/// # assert_eq!(option_page.offset(), 50);
///
/// # }
/// ```
#[derive(Debug)]
pub struct Page {
    pub(crate) page: u32,
    pub(crate) limit: u32,
    pub(crate) total: u32,
}

impl Page {
    /// Create a new Page instance
    pub fn new() -> Self {
        Page {
            page: 0,
            limit: DEFAULT_LIMIT,
            total: 0,
        }
    }
    /// Update current page to the next page
    pub fn next(&mut self) {
        // Don't overflow the page number, reset to 0
        if self.page == u32::MAX {
            self.page = 0;
        } else {
            self.page += 1;
        }
    }
    /// Update current page to the previous page
    pub fn prev(&mut self) {
        if self.page > 0 {
            self.page -= 1;
        }
    }
    /// Page number
    pub fn page(&self) -> u32 {
        self.page
    }
    /// Limit the rows accessed
    pub fn limit(&self) -> u32 {
        self.limit
    }
    /// Offset for the query
    pub fn offset(&self) -> u32 {
        if self.page == u32::MAX {
            0
        } else {
            self.page * self.limit
        }
    }
    /// Total number of pages
    pub fn pages(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            (self.total as f64 / self.limit as f64).ceil() as u32
        }
    }
    /// Get total number of rows
    pub fn total(&self) -> u32 {
        self.total
    }
    /// Set the total number of rows
    pub fn set_total(&mut self, total: u32) {
        self.total = total;
    }

    /// Get the maximum number of pages based on the total number of rows
    pub fn max(&self) -> u32 {
        if self.total == 0 {
            0
        } else {
            (self.total as f64 / self.limit as f64).ceil() as u32
        }
    }
}

impl Default for Page {
    fn default() -> Self {
        Page {
            page: u32::MAX,
            limit: DEFAULT_LIMIT,
            total: 0,
        }
    }
}

impl From<(u32, u32)> for Page {
    fn from(p: (u32, u32)) -> Self {
        let limit = if p.1 > DEFAULT_LIMIT {
            DEFAULT_LIMIT
        } else {
            p.1
        };
        Page {
            page: p.0,
            limit,
            ..Default::default()
        }
    }
}

impl From<(Option<u32>, Option<u32>)> for Page {
    fn from(value: (Option<u32>, Option<u32>)) -> Self {
        let mut page = Page::new();
        if let Some(p) = value.0 {
            page.page = p;
        }
        if let Some(l) = value.1 {
            if l > DEFAULT_LIMIT {
                page.limit = DEFAULT_LIMIT;
            } else {
                page.limit = l;
            }
        }
        page
    }
}

/// Implement From for Page (page, limit, total)
impl From<(Option<u32>, Option<u32>, u32)> for Page {
    fn from(value: (Option<u32>, Option<u32>, u32)) -> Self {
        let mut page = Page::new();
        if let Some(p) = value.0 {
            page.page = p;
        }
        if let Some(l) = value.1 {
            if l > DEFAULT_LIMIT {
                page.limit = DEFAULT_LIMIT;
            } else {
                page.limit = l;
            }
        }
        page.total = value.2;
        page
    }
}

impl From<u32> for Page {
    fn from(value: u32) -> Self {
        Page {
            page: value,
            limit: DEFAULT_LIMIT,
            ..Default::default()
        }
    }
}
