#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl Rect {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self { x, y, width, height }
    }

    pub fn from_tuple((x, y, width, height): (i32, i32, i32, i32)) -> Self {
        Self { x, y, width, height }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LayoutType {
    LeftHalf,
    RightHalf,
    TopHalf,
    BottomHalf,
    Fullscreen,
    Centered,
}

#[derive(Debug, Clone)]
pub struct LayoutEngine {
    gap: i32,
    margin: i32,
}

impl LayoutEngine {
    pub fn new(gap: i32, margin: i32) -> Self {
        Self { gap, margin }
    }

    pub fn calculate(&self, layout: LayoutType, monitor_area: Rect) -> Rect {
        let mx = monitor_area.x + self.margin;
        let my = monitor_area.y + self.margin;
        let mw = (monitor_area.width - 2 * self.margin).max(0);
        let mh = (monitor_area.height - 2 * self.margin).max(0);

        if mw <= 0 || mh <= 0 {
            return monitor_area;
        }

        match layout {
            LayoutType::LeftHalf => {
                let half_w = mw / 2 - self.gap / 2;
                Rect::new(mx, my, half_w.max(1), mh)
            }
            LayoutType::RightHalf => {
                let half_w = mw / 2 - self.gap / 2;
                Rect::new(mx + mw / 2 + self.gap / 2, my, half_w.max(1), mh)
            }
            LayoutType::TopHalf => {
                let half_h = mh / 2 - self.gap / 2;
                Rect::new(mx, my, mw, half_h.max(1))
            }
            LayoutType::BottomHalf => {
                let half_h = mh / 2 - self.gap / 2;
                Rect::new(mx, my + mh / 2 + self.gap / 2, mw, half_h.max(1))
            }
            LayoutType::Fullscreen => Rect::new(mx, my, mw, mh),
            LayoutType::Centered => {
                let cw = ((mw as f64) * 0.75) as i32;
                let ch = ((mh as f64) * 0.75) as i32;
                let cx = mx + (mw - cw) / 2;
                let cy = my + (mh - ch) / 2;
                Rect::new(cx, cy, cw.max(1), ch.max(1))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn monitor_1920x1080() -> Rect {
        Rect::new(0, 0, 1920, 1040)
    }

    #[test]
    fn test_left_half_no_gap() {
        let engine = LayoutEngine::new(0, 0);
        let result = engine.calculate(LayoutType::LeftHalf, monitor_1920x1080());
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
        assert_eq!(result.width, 960);
        assert_eq!(result.height, 1040);
    }

    #[test]
    fn test_right_half_no_gap() {
        let engine = LayoutEngine::new(0, 0);
        let result = engine.calculate(LayoutType::RightHalf, monitor_1920x1080());
        assert_eq!(result.x, 960);
        assert_eq!(result.y, 0);
        assert_eq!(result.width, 960);
        assert_eq!(result.height, 1040);
    }

    #[test]
    fn test_fullscreen_no_gap() {
        let engine = LayoutEngine::new(0, 0);
        let result = engine.calculate(LayoutType::Fullscreen, monitor_1920x1080());
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
        assert_eq!(result.width, 1920);
        assert_eq!(result.height, 1040);
    }

    #[test]
    fn test_centered() {
        let engine = LayoutEngine::new(0, 0);
        let result = engine.calculate(LayoutType::Centered, monitor_1920x1080());
        assert_eq!(result.width, 1440);
        assert_eq!(result.height, 780);
        assert_eq!(result.x, 240);
        assert_eq!(result.y, 130);
    }

    #[test]
    fn test_left_half_with_gap() {
        let engine = LayoutEngine::new(8, 0);
        let result = engine.calculate(LayoutType::LeftHalf, monitor_1920x1080());
        assert_eq!(result.width, 956);
        assert_eq!(result.height, 1040);
    }

    #[test]
    fn test_left_half_with_margin() {
        let engine = LayoutEngine::new(0, 10);
        let result = engine.calculate(LayoutType::LeftHalf, monitor_1920x1080());
        assert_eq!(result.x, 10);
        assert_eq!(result.y, 10);
        assert_eq!(result.width, 950);
        assert_eq!(result.height, 1020);
    }

    #[test]
    fn test_top_half() {
        let engine = LayoutEngine::new(8, 0);
        let result = engine.calculate(LayoutType::TopHalf, monitor_1920x1080());
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
        assert_eq!(result.width, 1920);
        assert_eq!(result.height, 516);
    }

    #[test]
    fn test_bottom_half() {
        let engine = LayoutEngine::new(8, 0);
        let result = engine.calculate(LayoutType::BottomHalf, monitor_1920x1080());
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 524);
        assert_eq!(result.width, 1920);
        assert_eq!(result.height, 516);
    }
}
