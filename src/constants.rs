// Layout and positioning constants for the graph view

/// Minimum gap between nodes in a column
pub const MIN_NODE_GAP: f32 = 20.0;

/// Top margin
pub const TOP_MARGIN: f32 = 20.0;

/// Minimum gap required when placing new nodes between existing ones
pub const MIN_PLACEMENT_GAP: f32 = 2.0 * MIN_NODE_GAP + 80.0;

/// Width threshold for considering nodes to be in the same column
pub const COLUMN_WIDTH: f32 = 200.0;

/// Canvas size for the graph view
pub const CANVAS_SIZE: f64 = 5000.0;
