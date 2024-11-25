//! # GeekORM Pages

/// Default limit for max page size
const DEFAULT_LIMIT: u32 = 100;

/// Pagination struct
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
/// // Create a new Pagination instance
/// let mut page = Pagination::new();
/// # assert_eq!(page.page(), 0);
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
/// let page_max = Pagination::from((1, 10_000));
/// # assert_eq!(page_max.limit(), 100);
///
/// let option_page = Pagination::from((Some(5), Some(10)));
/// # assert_eq!(option_page.page(), 5);
/// # assert_eq!(option_page.limit(), 10);
/// # assert_eq!(option_page.offset(), 50);
///
/// # }
/// ```
#[derive(Debug)]
pub struct Pagination {
    pub(crate) page: u32,
    pub(crate) limit: u32,
}

impl Pagination {
    /// Create a new Pagination instance
    pub fn new() -> Self {
        Pagination {
            page: 0,
            limit: DEFAULT_LIMIT,
        }
    }
    /// Update current page to the next page
    pub fn next(&mut self) {
        self.page += 1;
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
        self.page * self.limit
    }
}

impl Default for Pagination {
    fn default() -> Self {
        Pagination {
            page: 0,
            limit: DEFAULT_LIMIT,
        }
    }
}

impl From<(u32, u32)> for Pagination {
    fn from(p: (u32, u32)) -> Self {
        let limit = if p.1 > DEFAULT_LIMIT {
            DEFAULT_LIMIT
        } else {
            p.1
        };
        Pagination { page: p.0, limit }
    }
}

impl From<(Option<u32>, Option<u32>)> for Pagination {
    fn from(value: (Option<u32>, Option<u32>)) -> Self {
        let mut page = Pagination::new();
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

impl From<u32> for Pagination {
    fn from(value: u32) -> Self {
        Pagination {
            page: value,
            limit: DEFAULT_LIMIT,
        }
    }
}
