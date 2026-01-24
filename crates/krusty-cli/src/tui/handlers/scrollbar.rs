//! Scrollbar handling
//!
//! Handles scrollbar click and drag operations for messages, input, and block scrollbars.

use crate::tui::app::App;
use crate::tui::blocks::BlockType;
use crate::tui::state::DragTarget;

impl App {
    /// Handle plan sidebar scrollbar click - jump to position
    pub fn handle_plan_sidebar_scrollbar_click(
        &mut self,
        click_y: u16,
        area: ratatui::layout::Rect,
    ) {
        self.plan_sidebar.handle_scrollbar_click(click_y, area);
    }

    /// Handle scrollbar drag - routes to appropriate scrollbar based on drag target
    ///
    /// Returns true if a scrollbar drag was handled.
    pub fn handle_scrollbar_drag(&mut self, y: u16) -> bool {
        match self.scroll_system.layout.dragging_scrollbar {
            Some(DragTarget::Messages(drag)) => {
                let new_offset = drag.calculate_offset(y);
                self.scroll_system.scroll.scroll_to_line(new_offset);
                true
            }
            Some(DragTarget::Input(drag)) => {
                let new_offset = drag.calculate_offset(y);
                self.input.set_viewport_offset(new_offset);
                true
            }
            Some(DragTarget::PlanSidebar) => {
                if let Some(area) = self.scroll_system.layout.plan_sidebar_scrollbar_area {
                    self.handle_plan_sidebar_scrollbar_click(y, area);
                }
                true
            }
            Some(DragTarget::Block(drag)) => {
                if let Some(offset) = drag.calculate_offset(y) {
                    match drag.block_type {
                        BlockType::Thinking => {
                            if let Some(block) = self.blocks.thinking.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::ToolResult => {
                            if let Some(block) = self.blocks.tool_result.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::Bash => {
                            if let Some(block) = self.blocks.bash.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::Read => {
                            if let Some(block) = self.blocks.read.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::Edit => {
                            if let Some(block) = self.blocks.edit.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::Write => {
                            if let Some(block) = self.blocks.write.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::WebSearch => {
                            if let Some(block) = self.blocks.web_search.get_mut(drag.index) {
                                block.set_scroll_offset(offset);
                            }
                        }
                        BlockType::TerminalPane => {
                            // Terminal panes don't use this scrollbar system
                        }
                        BlockType::Explore => {
                            // Explore blocks don't use this scrollbar system
                        }
                        BlockType::Build => {
                            // Build blocks don't use this scrollbar system
                        }
                    }
                }
                true
            }
            None => false,
        }
    }
}
