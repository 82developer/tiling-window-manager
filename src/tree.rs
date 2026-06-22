use crate::layout::Rect;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection {
    Horizontal,
    Vertical,
}

impl SplitDirection {
    pub fn toggle(self) -> Self {
        match self {
            Self::Horizontal => Self::Vertical,
            Self::Vertical => Self::Horizontal,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Layout {
    SplitH,
    SplitV,
    Monocle,
    Stacking,
}

#[derive(Debug, Clone)]
pub struct FocusDirection {
    pub dx: i32,
    pub dy: i32,
}

impl FocusDirection {
    pub const LEFT: Self = Self { dx: -1, dy: 0 };
    pub const RIGHT: Self = Self { dx: 1, dy: 0 };
    pub const UP: Self = Self { dx: 0, dy: -1 };
    pub const DOWN: Self = Self { dx: 0, dy: 1 };
}

#[derive(Debug, Clone)]
struct Node {
    id: usize,
    parent: Option<usize>,
    children: Vec<usize>,
    hwnd: Option<isize>,
    direction: SplitDirection,
}

#[derive(Debug, Clone)]
pub struct ContainerTree {
    nodes: Vec<Node>,
    root_id: usize,
    focused_id: usize,
    next_id: usize,
    layout: Layout,
    monocle_focused: Option<usize>,
}

impl ContainerTree {
    pub fn new() -> Self {
        let root = Node {
            id: 0,
            parent: None,
            children: Vec::new(),
            hwnd: None,
            direction: SplitDirection::Vertical,
        };
        Self {
            nodes: vec![root],
            root_id: 0,
            focused_id: 0,
            next_id: 1,
            layout: Layout::SplitV,
            monocle_focused: None,
        }
    }

    fn allocate_id(&mut self) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        id
    }

    fn add_leaf(&mut self, hwnd: isize, parent: Option<usize>) -> usize {
        let id = self.allocate_id();
        let node = Node {
            id,
            parent,
            children: Vec::new(),
            hwnd: Some(hwnd),
            direction: SplitDirection::Vertical,
        };
        self.nodes.push(node);
        id
    }

    fn add_container(&mut self, direction: SplitDirection, parent: Option<usize>) -> usize {
        let id = self.allocate_id();
        let node = Node {
            id,
            parent,
            children: Vec::new(),
            hwnd: None,
            direction,
        };
        self.nodes.push(node);
        id
    }

    pub fn focused_hwnd(&self) -> Option<isize> {
        self.nodes.get(self.focused_id).and_then(|n| n.hwnd)
    }

    pub fn root_id(&self) -> usize {
        self.root_id
    }

    pub fn focused_id(&self) -> usize {
        self.focused_id
    }

    pub fn set_layout(&mut self, layout: Layout) {
        self.layout = layout;
        if layout == Layout::Monocle {
            self.monocle_focused = Some(self.focused_id);
        } else {
            self.monocle_focused = None;
        }
    }

    pub fn get_layout(&self) -> Layout {
        self.layout
    }

    /// Make the focused leaf its own container so the next window splits inside it.
    fn ensure_focused_is_container(&mut self) {
        let focused = &self.nodes[self.focused_id];
        if focused.hwnd.is_none() {
            return;
        }
        if focused.parent.is_none() {
            return;
        }

        let parent_id = focused.parent.unwrap();
        let direction = self.nodes[self.focused_id].direction;
        let container_id = self.add_container(direction, Some(parent_id));

        self.nodes[self.focused_id].parent = Some(container_id);
        self.nodes[container_id].children.push(self.focused_id);

        let parent = &mut self.nodes[parent_id];
        if let Some(pos) = parent.children.iter().position(|c| *c == self.focused_id) {
            parent.children[pos] = container_id;
        }

        self.focused_id = container_id;

        tracing::info!(
            "created container {} with {:?} around leaf, now focused",
            container_id, direction
        );
    }

    pub fn toggle_split(&mut self) {
        if self.nodes[self.focused_id].hwnd.is_some() {
            if let Some(pid) = self.nodes[self.focused_id].parent {
                self.focused_id = pid;
            }
        }
        let dir = self.nodes[self.focused_id].direction.toggle();
        self.nodes[self.focused_id].direction = dir;
        tracing::info!("container {} split toggled to {:?}", self.focused_id, dir);
    }

    pub fn set_split_horizontal(&mut self) {
        self.ensure_focused_is_container();
        self.nodes[self.focused_id].direction = SplitDirection::Horizontal;
        tracing::info!("container {} set to Horizontal", self.focused_id);
    }

    pub fn set_split_vertical(&mut self) {
        self.ensure_focused_is_container();
        self.nodes[self.focused_id].direction = SplitDirection::Vertical;
        tracing::info!("container {} set to Vertical", self.focused_id);
    }

    pub fn get_split_direction(&self, node_id: usize) -> SplitDirection {
        self.nodes
            .get(node_id)
            .map(|n| n.direction)
            .unwrap_or(SplitDirection::Vertical)
    }

    /// Add a window next to the focused node. The new window becomes a sibling.
    /// The parent container's split direction determines placement.
    pub fn add_window(&mut self, hwnd: isize) -> usize {
        if let Some(existing_id) = self.find_window(hwnd) {
            self.focused_id = existing_id;
            return existing_id;
        }

        let focused = &self.nodes[self.focused_id];

        if focused.parent.is_none() {
            // Focused is root or orphan - simple add to root
            let new_id = self.add_leaf(hwnd, Some(self.root_id));
            self.nodes[self.root_id].children.push(new_id);
            self.focused_id = new_id;
            return new_id;
        }

        let parent_id = focused.parent.unwrap();

        if focused.hwnd.is_some() {
            // Focused is a leaf window. Create a container to hold both.
            let parent_direction = self.nodes[parent_id].direction;
            let container_id = self.add_container(parent_direction, Some(parent_id));

            // Move focused window into new container
            self.nodes[self.focused_id].parent = Some(container_id);
            self.nodes[container_id].children.push(self.focused_id);

            // Add new window as sibling
            let new_id = self.add_leaf(hwnd, Some(container_id));
            self.nodes[container_id].children.push(new_id);

            // Replace focused in parent's children list
            let parent = &mut self.nodes[parent_id];
            if let Some(pos) = parent.children.iter().position(|c| *c == self.focused_id) {
                parent.children[pos] = container_id;
            } else {
                parent.children.push(container_id);
            }

            self.focused_id = new_id;
            tracing::debug!(
                "container {} created with direction {:?}, window {} added next to {}",
                container_id,
                parent_direction,
                hwnd,
                self.focused_id
            );
            new_id
        } else {
            // Focused is a container. Add child directly.
            let container_id = self.focused_id;
            let direction = self.nodes[container_id].direction;
            let new_id = self.add_leaf(hwnd, Some(container_id));
            self.nodes[container_id].children.push(new_id);
            self.focused_id = new_id;
            tracing::debug!(
                "window {} added to container {} with direction {:?}",
                hwnd,
                container_id,
                direction
            );
            new_id
        }
    }

    pub fn remove_window(&mut self, hwnd: isize) {
        let target_id = match self.find_window(hwnd) {
            Some(id) => id,
            None => return,
        };

        let parent_id = self.nodes[target_id].parent;

        // Remove from parent's children
        if let Some(pid) = parent_id {
            self.nodes[pid].children.retain(|c| *c != target_id);

            // If focus was on this node, move focus to parent or sibling
            if self.focused_id == target_id {
                if self.nodes[pid].children.is_empty() {
                    self.focused_id = pid;
                } else {
                    self.focused_id = self.nodes[pid].children[0];
                }
            }

            // Clean up empty containers (except root)
            if pid != self.root_id
                && self.nodes[pid].children.is_empty()
                && self.nodes[pid].hwnd.is_none()
            {
                self.cleanup_empty_container(pid);
            }
        }

        // Actually remove the node
        self.nodes[target_id].hwnd = None;
        self.nodes[target_id].children.clear();
    }

    fn cleanup_empty_container(&mut self, container_id: usize) {
        let grandparent_id = self.nodes[container_id].parent;

        // Remove from grandparent
        if let Some(gpid) = grandparent_id {
            self.nodes[gpid].children.retain(|c| *c != container_id);
            if self.focused_id == container_id {
                self.focused_id = gpid;
            }
            // Recursive cleanup
            if gpid != self.root_id
                && self.nodes[gpid].children.is_empty()
                && self.nodes[gpid].hwnd.is_none()
            {
                self.cleanup_empty_container(gpid);
            }
        }
    }

    pub fn find_window(&self, hwnd: isize) -> Option<usize> {
        self.nodes
            .iter()
            .find(|n| n.hwnd == Some(hwnd))
            .map(|n| n.id)
    }

    pub fn focus_container(&mut self, id: usize) {
        if id < self.nodes.len() {
            self.focused_id = id;
        }
    }

    pub fn all_windows(&self) -> Vec<isize> {
        self.nodes.iter().filter_map(|n| n.hwnd).collect()
    }

    pub fn visible_windows(&self) -> Vec<isize> {
        if self.layout == Layout::Monocle {
            if let Some(id) = self.monocle_focused {
                return self.nodes.get(id).and_then(|n| n.hwnd).into_iter().collect();
            }
            return Vec::new();
        }
        self.all_windows()
    }

    /// Recursive layout calculation
    pub fn calculate_layouts(&self, area: Rect, gap: i32) -> Vec<(isize, Rect)> {
        if self.layout == Layout::Monocle || self.layout == Layout::Stacking {
            return self
                .visible_windows()
                .into_iter()
                .map(|hwnd| (hwnd, area))
                .collect();
        }

        let mut results = Vec::new();
        self.calculate_node(self.root_id, area, gap, &mut results);
        results
    }

    fn calculate_node(
        &self,
        node_id: usize,
        area: Rect,
        gap: i32,
        results: &mut Vec<(isize, Rect)>,
    ) {
        let node = match self.nodes.get(node_id) {
            Some(n) => n,
            None => return,
        };

        if let Some(hwnd) = node.hwnd {
            results.push((hwnd, area));
            return;
        }

        let children: Vec<usize> = node.children.iter().copied().collect();
        let leaf_children: Vec<usize> = children
            .iter()
            .filter(|id| self.is_leaf_or_has_windows(**id))
            .copied()
            .collect();

        if leaf_children.is_empty() {
            return;
        }

        let n = leaf_children.len();
        if n == 1 {
            self.calculate_node(leaf_children[0], area, gap, results);
            return;
        }

        match node.direction {
            SplitDirection::Horizontal => {
                let cell_width = (area.width - gap * (n as i32 - 1)) / n as i32;
                for (i, &child_id) in leaf_children.iter().enumerate() {
                    let x = area.x + i as i32 * (cell_width + gap);
                    let child_area = Rect::new(x, area.y, cell_width.max(1), area.height);
                    self.calculate_node(child_id, child_area, gap, results);
                }
            }
            SplitDirection::Vertical => {
                let cell_height = (area.height - gap * (n as i32 - 1)) / n as i32;
                for (i, &child_id) in leaf_children.iter().enumerate() {
                    let y = area.y + i as i32 * (cell_height + gap);
                    let child_area = Rect::new(area.x, y, area.width, cell_height.max(1));
                    self.calculate_node(child_id, child_area, gap, results);
                }
            }
        }
    }

    fn is_leaf_or_has_windows(&self, node_id: usize) -> bool {
        match self.nodes.get(node_id) {
            Some(n) => n.hwnd.is_some() || !n.children.is_empty(),
            None => false,
        }
    }

    /// Spatial focus navigation across all windows in the tree
    pub fn focus_direction(&mut self, dir: &FocusDirection, area: Rect) -> Option<usize> {
        let current_hwnd = self.focused_hwnd()?;
        let layouts = self.calculate_layouts(area, 0);

        let current_rect = layouts
            .iter()
            .find(|(hw, _)| *hw == current_hwnd)
            .map(|(_, r)| *r)?;

        let mut best_id: Option<usize> = None;
        let mut best_dist: f64 = f64::MAX;

        for (target_hwnd, rect) in &layouts {
            if *target_hwnd == current_hwnd {
                continue;
            }

            let cx1 = current_rect.x as f64 + current_rect.width as f64 / 2.0;
            let cy1 = current_rect.y as f64 + current_rect.height as f64 / 2.0;
            let cx2 = rect.x as f64 + rect.width as f64 / 2.0;
            let cy2 = rect.y as f64 + rect.height as f64 / 2.0;

            let valid = match (dir.dx, dir.dy) {
                (-1, 0) => rect.x + rect.width <= current_rect.x,
                (1, 0) => rect.x >= current_rect.x + current_rect.width,
                (0, -1) => rect.y + rect.height <= current_rect.y,
                (0, 1) => rect.y >= current_rect.y + current_rect.height,
                _ => false,
            };

            if valid {
                let dist = ((cx1 - cx2).powi(2) + (cy1 - cy2).powi(2)) as f64;
                if dist < best_dist {
                    best_dist = dist;
                    best_id = self.find_window(*target_hwnd);
                }
            }
        }

        best_id
    }

    /// Move the focused window within the tree in the given direction.
    /// This swaps positions by restructuring the tree, not by pixel movement.
    pub fn move_focused_in_direction(&mut self, dir: &FocusDirection, area: Rect) -> bool {
        let focused_hwnd = match self.focused_hwnd() {
            Some(h) => h,
            None => return false,
        };

        let layouts = self.calculate_layouts(area, 0);
        let current_rect = match layouts.iter().find(|(hw, _)| *hw == focused_hwnd).map(|(_, r)| *r) {
            Some(r) => r,
            None => return false,
        };

        let mut target_hwnd: Option<isize> = None;
        let mut best_dist: f64 = f64::MAX;

        for (hw, rect) in &layouts {
            if *hw == focused_hwnd {
                continue;
            }
            let cx1 = current_rect.x as f64 + current_rect.width as f64 / 2.0;
            let cy1 = current_rect.y as f64 + current_rect.height as f64 / 2.0;
            let cx2 = rect.x as f64 + rect.width as f64 / 2.0;
            let cy2 = rect.y as f64 + rect.height as f64 / 2.0;

            let valid = match (dir.dx, dir.dy) {
                (-1, 0) => rect.x + rect.width <= current_rect.x,
                (1, 0) => rect.x >= current_rect.x + current_rect.width,
                (0, -1) => rect.y + rect.height <= current_rect.y,
                (0, 1) => rect.y >= current_rect.y + current_rect.height,
                _ => false,
            };

            if valid {
                let dist = ((cx1 - cx2).powi(2) + (cy1 - cy2).powi(2)) as f64;
                if dist < best_dist {
                    best_dist = dist;
                    target_hwnd = Some(*hw);
                }
            }
        }

        let target_hwnd = match target_hwnd {
            Some(h) => h,
            None => return false,
        };

        let target_id = match self.find_window(target_hwnd) {
            Some(id) => id,
            None => return false,
        };

        self.swap_nodes(self.focused_id, target_id);
        true
    }

    /// Swap two nodes in the tree by exchanging their positions.
    fn swap_nodes(&mut self, a_id: usize, b_id: usize) {
        if a_id == b_id {
            return;
        }

        let a_parent = self.nodes[a_id].parent;
        let b_parent = self.nodes[b_id].parent;

        if a_parent.is_none() || b_parent.is_none() {
            return;
        }

        let a_parent = a_parent.unwrap();
        let b_parent = b_parent.unwrap();

        // Update parent references
        self.nodes[a_id].parent = Some(b_parent);
        self.nodes[b_id].parent = Some(a_parent);

        // Update children lists in parents
        if a_parent == b_parent {
            // Same parent: just swap positions in children list
            let parent = &mut self.nodes[a_parent];
            let a_pos = parent.children.iter().position(|c| *c == a_id);
            let b_pos = parent.children.iter().position(|c| *c == b_id);
            if let (Some(pa), Some(pb)) = (a_pos, b_pos) {
                parent.children.swap(pa, pb);
            }
        } else {
            // Different parents: replace in each parent's children
            if let Some(parent) = self.nodes.get_mut(a_parent) {
                if let Some(pos) = parent.children.iter().position(|c| *c == a_id) {
                    parent.children[pos] = b_id;
                }
            }
            if let Some(parent) = self.nodes.get_mut(b_parent) {
                if let Some(pos) = parent.children.iter().position(|c| *c == b_id) {
                    parent.children[pos] = a_id;
                }
            }
        }

        tracing::info!("swapped nodes {} and {}", a_id, b_id);
    }

    pub fn focus_next_leaf(&mut self) {
        let windows: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| n.hwnd.is_some())
            .map(|n| n.id)
            .collect();

        if windows.is_empty() {
            return;
        }

        let pos = windows
            .iter()
            .position(|id| *id == self.focused_id)
            .unwrap_or(0);
        let next = (pos + 1) % windows.len();
        self.focused_id = windows[next];
    }

    pub fn focus_prev_leaf(&mut self) {
        let windows: Vec<usize> = self
            .nodes
            .iter()
            .filter(|n| n.hwnd.is_some())
            .map(|n| n.id)
            .collect();

        if windows.is_empty() {
            return;
        }

        let pos = windows
            .iter()
            .position(|id| *id == self.focused_id)
            .unwrap_or(0);
        let prev = if pos == 0 {
            windows.len() - 1
        } else {
            pos - 1
        };
        self.focused_id = windows[prev];
    }

    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    pub fn window_count(&self) -> usize {
        self.nodes.iter().filter(|n| n.hwnd.is_some()).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_area() -> Rect {
        Rect::new(0, 0, 1920, 1040)
    }

    #[test]
    fn test_new_tree_empty() {
        let tree = ContainerTree::new();
        assert_eq!(tree.window_count(), 0);
        assert_eq!(tree.node_count(), 1); // root only
    }

    #[test]
    fn test_add_single_window() {
        let mut tree = ContainerTree::new();
        let id = tree.add_window(100);
        assert_eq!(tree.window_count(), 1);
        assert_eq!(tree.focused_hwnd(), Some(100));
    }

    #[test]
    fn test_add_two_windows_vertical() {
        let mut tree = ContainerTree::new();
        tree.set_split_vertical();
        tree.add_window(100);
        tree.add_window(200);

        let area = make_area();
        let layouts = tree.calculate_layouts(area, 0);
        assert_eq!(layouts.len(), 2);

        // In vertical split, first window should be on top
        let (hwnd1, rect1) = layouts[0];
        let (hwnd2, rect2) = layouts[1];

        assert_eq!(rect1.y, 0);
        assert_eq!(rect2.y, 520); // half the height
        assert_eq!(rect1.height, 520);
        assert_eq!(rect2.height, 520);
    }

    #[test]
    fn test_add_two_windows_horizontal() {
        let mut tree = ContainerTree::new();
        tree.set_split_horizontal();
        tree.add_window(100);
        tree.add_window(200);

        let area = make_area();
        let layouts = tree.calculate_layouts(area, 0);
        assert_eq!(layouts.len(), 2);

        let (_, rect1) = layouts[0];
        let (_, rect2) = layouts[1];

        assert_eq!(rect1.x, 0);
        assert_eq!(rect2.x, 960);
        assert_eq!(rect1.width, 960);
        assert_eq!(rect2.width, 960);
    }

    #[test]
    fn test_remove_window_cleans_up() {
        let mut tree = ContainerTree::new();
        tree.set_split_vertical();
        tree.add_window(100);
        tree.add_window(200);

        assert_eq!(tree.window_count(), 2);

        tree.remove_window(100);
        assert_eq!(tree.window_count(), 1);
        assert_eq!(tree.all_windows(), vec![200]);
    }

    #[test]
    fn test_toggle_split() {
        let mut tree = ContainerTree::new();
        assert_eq!(tree.get_split_direction(tree.root_id), SplitDirection::Vertical);

        tree.toggle_split();
        assert_eq!(tree.get_split_direction(tree.root_id), SplitDirection::Horizontal);

        tree.toggle_split();
        assert_eq!(tree.get_split_direction(tree.root_id), SplitDirection::Vertical);
    }

    #[test]
    fn test_monocle_visible() {
        let mut tree = ContainerTree::new();
        tree.add_window(100);
        tree.add_window(200);

        assert_eq!(tree.visible_windows().len(), 2);

        tree.set_layout(Layout::Monocle);
        assert_eq!(tree.visible_windows().len(), 1);
    }

    #[test]
    fn test_nested_split() {
        // Create a complex tree:
        // Root(V) → [Container(H) → [100, 200], Container(V) → [300, 400]]
        let mut tree = ContainerTree::new();

        tree.set_split_horizontal();
        tree.add_window(100);
        tree.add_window(200);

        // Focus on root and add more vertical
        tree.focus_container(tree.root_id);
        tree.set_split_vertical();
        tree.add_window(300);
        tree.add_window(400);

        let area = make_area();
        let layouts = tree.calculate_layouts(area, 0);
        assert_eq!(layouts.len(), 4);
    }

    #[test]
    fn test_focus_direction_right() {
        let mut tree = ContainerTree::new();
        tree.set_split_horizontal();
        tree.add_window(100);
        tree.add_window(200);

        let area = make_area();
        let target = tree.focus_direction(
            &FocusDirection { dx: 1, dy: 0 },
            area,
        );

        // Focus first window (100), direction RIGHT should find 200
        tree.focus_container(tree.find_window(100).unwrap());
        let target = tree.focus_direction(&FocusDirection::RIGHT, area);
        assert_eq!(target, tree.find_window(200));
    }
}
