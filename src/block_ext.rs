use crate::*;

impl Block {
    pub(crate) fn append_stmt(mut self, stmt: Stmt) -> Self {
        self.statements.push(stmt);
        self
    }

    pub(crate) fn push_expr(&mut self, expr: Expr) -> &mut Self {
        self.statements.push(Stmt::Expr(expr));
        self
    }

    pub(crate) fn extend_with(mut self, expr_vec: Vec<Expr>) -> Self {
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
        self
    }

    pub(crate) fn append_string_sequence(&mut self, sequence: &str) -> Result<()> {
        let mut expr_vec = vec![];
        expr_vec.append_string_sequence(sequence)?;
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
        Ok(())
    }

    pub(crate) fn sleep_for(mut self, duration: time::Duration) -> Self {
        self.statements.push(Stmt::Expr(Expr::SleepAction(duration)));
        self
    }
}

impl Expr {
    pub(crate) fn map_key_click_block(from: KeyClickActionWithMods, mut to: Block) -> Self {
        // release modifiers from the trigger
        if from.modifiers.ctrl {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })));
        }
        if from.modifiers.alt {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })));
        }
        if from.modifiers.shift {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })));
        }
        if from.modifiers.meta {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })));
        }

        Expr::KeyMapping(vec![
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_DOWN, from.modifiers), to },
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_REPEAT, from.modifiers), to: Block::new() }, // stub
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_UP, from.modifiers), to: Block::new() }, // stuv
        ])
    }

    pub(crate) fn map_key_block(from: KeyActionWithMods, mut to: Block) -> Self {
        // release modifiers from the trigger
        if from.modifiers.ctrl {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })));
        }
        if from.modifiers.alt {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })));
        }
        if from.modifiers.shift {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })));
        }
        if from.modifiers.meta {
            to.statements.insert(0, Stmt::Expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })));
            to.statements.insert(1, Stmt::Expr(Expr::EatKeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })));
        }

        Expr::KeyMapping(vec![KeyMapping { from, to }])
    }

    // PROTO
    pub(crate) fn map_key_action_action(from: KeyActionWithMods, to: KeyActionWithMods) -> Self {
        let mut block = Block::new();

        if from.modifiers.ctrl && !to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
        if from.modifiers.alt && !to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
        if from.modifiers.shift && !to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
        if from.modifiers.meta && !to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

        if !from.modifiers.ctrl && to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
        if !from.modifiers.alt && to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
        if !from.modifiers.shift && to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
        if !from.modifiers.meta && to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

        block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: to.value }));

        // revert to original
        if !from.modifiers.ctrl && to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
        if !from.modifiers.alt && to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
        if !from.modifiers.shift && to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
        if !from.modifiers.meta && to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

        if from.modifiers.ctrl && !to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
        if from.modifiers.alt && !to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
        if from.modifiers.shift && !to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
        if from.modifiers.meta && !to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

        Expr::KeyMapping(vec![KeyMapping { from, to: block }])
    }

    pub(crate) fn map_key_action_click(from: KeyActionWithMods, to: KeyClickActionWithMods) -> Self {
        let mut block = Block::new();

        if from.modifiers.ctrl && !to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
        if from.modifiers.alt && !to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
        if from.modifiers.shift && !to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
        if from.modifiers.meta && !to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

        if !from.modifiers.ctrl && to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
        if !from.modifiers.alt && to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
        if !from.modifiers.shift && to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
        if !from.modifiers.meta && to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

        block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));
        block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

        // revert to original
        if !from.modifiers.ctrl && to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
        if !from.modifiers.alt && to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
        if !from.modifiers.shift && to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
        if !from.modifiers.meta && to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

        if from.modifiers.ctrl && !to.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
        if from.modifiers.alt && !to.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
        if from.modifiers.shift && !to.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
        if from.modifiers.meta && !to.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

        Expr::KeyMapping(vec![KeyMapping { from, to: block }])
    }

    pub(crate) fn map_key_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> Self {
        let mut mappings = vec![];
        {
            let mut block = Block::new();

            if from.modifiers.ctrl && !to.modifiers.ctrl {
                block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(*KEY_LEFT_CTRL, TYPE_UP)));
            }
            if from.modifiers.alt && !to.modifiers.alt {
                block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(*KEY_LEFT_ALT, TYPE_UP)));
            }
            if from.modifiers.shift && !to.modifiers.shift {
                block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(*KEY_LEFT_SHIFT, TYPE_UP)));
            }
            if from.modifiers.meta && !to.modifiers.meta {
                block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(*KEY_LEFT_META, TYPE_UP)));
            }

            if to.modifiers.ctrl && !from.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_DOWN })); }
            if to.modifiers.alt && !from.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_DOWN })); }
            if to.modifiers.shift && !from.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
            if to.modifiers.meta && !from.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_DOWN })); }

            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));

            mappings.push(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, to: block });
        }

        {
            let mut block = Block::new();
            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

            if to.modifiers.ctrl && !from.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_CTRL, value: TYPE_UP })); }
            if to.modifiers.alt && !from.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_ALT, value: TYPE_UP })); }
            if to.modifiers.shift && !from.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_SHIFT, value: TYPE_UP })); }
            if to.modifiers.meta && !from.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: *KEY_LEFT_META, value: TYPE_UP })); }

            mappings.push(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, to: block });
        }

        {
            let mut block = Block::new();
            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_REPEAT }));

            mappings.push(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers }, to: block });
        }
        Expr::KeyMapping(mappings)
    }
}

pub(crate) trait ExprVecExt {
    fn append_click(self, key: Key) -> Self;
    fn append_action(self, action: KeyAction) -> Self;
    fn sleep_for_millis(self, duration: u64) -> Self;
    fn append_string_sequence(&mut self, sequence: &str) -> Result<()>;
}

impl ExprVecExt for Vec<Expr> {
    fn append_click(mut self, key: Key) -> Self {
        self = self.append_action(KeyAction::new(key, TYPE_DOWN));
        self = self.append_action(KeyAction::new(key, TYPE_UP));
        self
    }

    fn append_action(mut self, action: KeyAction) -> Self {
        self.push(Expr::KeyAction(action));
        self
    }

    fn sleep_for_millis(self, duration: u64) -> Self {
        unimplemented!();
    }

    fn append_string_sequence(&mut self, sequence: &str) -> Result<()> {
        let mut it = sequence.chars();

        while let Some(ch) = it.next() {
            // special
            if ch == '{' {
                let special_char = it.by_ref().take_while(|&ch| ch != '}').collect::<String>();
                let seq = KEY_SEQ_LOOKUP.get(special_char.as_str())
                    .ok_or(anyhow!("failed to lookup key '{}'", special_char))?;
                self.extend(seq.iter().cloned());
                continue;
            }

            let seq = KEY_SEQ_LOOKUP.get(ch.to_string().as_str())
                .ok_or(anyhow!("failed to lookup key '{}'", ch))?;
            self.extend(seq.iter().cloned());
        }
        Ok(())
    }
}


