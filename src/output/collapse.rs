/// A collapsible region within the output.
#[derive(Debug)]
pub struct CollapseRegion {
    pub start: usize,
    pub end: usize,
    pub header: String,
    pub collapsed: bool,
}

pub struct CollapseManager {
    regions: Vec<CollapseRegion>,
    next_id: usize,
}

impl Default for CollapseManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CollapseManager {
    pub fn new() -> Self {
        Self {
            regions: Vec::new(),
            next_id: 0,
        }
    }

    /// Add a new collapsible region. Returns its ID.
    pub fn add_region(&mut self, start: usize, end: usize, header: &str) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.regions.push(CollapseRegion {
            start,
            end,
            header: header.to_string(),
            collapsed: false,
        });
        id
    }

    /// Toggle the collapse state of region `id`. Returns the new state.
    pub fn toggle(&mut self, id: usize) -> Option<bool> {
        let region = self.regions.get_mut(id)?;
        region.collapsed = !region.collapsed;
        Some(region.collapsed)
    }

    /// Check if `line_idx` falls inside a collapsed region.
    pub fn is_hidden(&self, line_idx: usize) -> bool {
        self.regions
            .iter()
            .any(|r| r.collapsed && line_idx > r.start && line_idx < r.end)
    }

    /// Return the region ID whose header is at `line_idx`, if any.
    pub fn region_at(&self, line_idx: usize) -> Option<usize> {
        self.regions
            .iter()
            .position(|r| r.start == line_idx && r.collapsed)
    }

    /// Get the header text for a collapsed region at `line_idx`.
    pub fn collapsed_header_at(&self, line_idx: usize) -> Option<&str> {
        self.regions
            .iter()
            .find(|r| r.start == line_idx && r.collapsed)
            .map(|r| r.header.as_str())
    }

    pub fn regions(&self) -> &[CollapseRegion] {
        &self.regions
    }

    /// Toggle the most recently added region. Returns the new state.
    pub fn toggle_last(&mut self) -> Option<bool> {
        if self.regions.is_empty() {
            return None;
        }
        let last_idx = self.regions.len() - 1;
        self.regions[last_idx].collapsed = !self.regions[last_idx].collapsed;
        Some(self.regions[last_idx].collapsed)
    }
}
