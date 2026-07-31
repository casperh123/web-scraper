pub struct PaginatedResult<T> {
    pub items: Vec<T>,
    pub total_items: u64,
    pub total_pages: u64,
}
