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

    pub(crate) fn replace_key_block(&mut self, from: KeyActionWithMods, mut to: Block) {
        self.attach_underlying_scope(&mut to);
        self.statements.push(Stmt::Expr(Expr::KeyMapping { 0: KeyMapping { from, to } }));
    }

    pub(crate) fn replace_key(&mut self, from: KeyActionWithMods, to: KeyAction) {
        let mut block = Block::new();
        let expr = Expr::KeyAction(to);
        let stmt = Stmt::Expr(expr);
        block.statements.push(stmt);
        self.replace_key_block(from, block);
    }

    pub(crate) fn extend_with(mut self, expr_vec: Vec<Expr>) -> Self {
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
        self
    }

    pub(crate) fn append_string_sequence(mut self, sequence: &str) -> Self {
        let expr_vec = vec![].append_string_sequence(sequence);
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
        self
    }

    pub(crate) fn sleep_for(mut self, duration: time::Duration) -> Self {
        self.statements.push(Stmt::Expr(Expr::SleepAction(duration)));
        self
    }
}

pub(crate) trait ExprVecExt {
    fn append_click(self, key: Key) -> Self;
    fn append_action(self, action: KeyAction) -> Self;
    fn sleep_for_millis(self, duration: u64) -> Self;
    fn append_string_sequence(self, sequence: &str) -> Self;
    fn map_key_click(&mut self, from: KeyClickActionWithMods, to: KeyClickActionWithMods);
    fn map_key_block(&mut self, from: KeyClickActionWithMods, to: Block);
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

    fn append_string_sequence(mut self, sequence: &str) -> Self {
        let mut it = sequence.chars();

        while let Some(ch) = it.next() {
            // special
            if ch == '{' {
                let special_char = it.by_ref().take_while(|&ch| ch != '}').collect::<String>();
                let seq = KEY_SEQ_LOOKUP.get(special_char.as_str())
                    .expect(format!("failed to lookup key '{}'", special_char).as_str());
                self.extend(seq.iter().cloned());
                continue;
            }

            let seq = KEY_SEQ_LOOKUP.get(ch.to_string().as_str())
                .expect(format!("failed to lookup key '{}'", ch).as_str());
            self.extend(seq.iter().cloned());
        }
        self
    }

    fn map_key_click(&mut self, from: KeyClickActionWithMods, to: KeyClickActionWithMods) {
        {
            let mut block = Block::new();

            if from.modifiers.ctrl && !to.modifiers.ctrl {
                block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(KEY_LEFT_CTRL, TYPE_UP)));
            }
            if from.modifiers.alt && !to.modifiers.alt {
                block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(KEY_LEFT_ALT, TYPE_UP)));
            }
            if from.modifiers.shift && !to.modifiers.shift {
                block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(KEY_LEFT_SHIFT, TYPE_UP)));
            }
            if from.modifiers.meta && !to.modifiers.meta {
                block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_UP }));
                block.push_expr(Expr::EatKeyAction(KeyAction::new(KEY_LEFT_META, TYPE_UP)));
            }

            if to.modifiers.ctrl && !from.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_DOWN })); }
            if to.modifiers.alt && !from.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_DOWN })); }
            if to.modifiers.shift && !from.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_DOWN })); }
            if to.modifiers.meta && !from.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_DOWN })); }

            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_DOWN }));

            self.push(Expr::KeyMapping(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_DOWN, modifiers: from.modifiers.clone() }, to: block }))
        }

        {
            let mut block = Block::new();
            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_UP }));

            if to.modifiers.ctrl && !from.modifiers.ctrl { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_CTRL, value: TYPE_UP })); }
            if to.modifiers.alt && !from.modifiers.alt { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_ALT, value: TYPE_UP })); }
            if to.modifiers.shift && !from.modifiers.shift { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_SHIFT, value: TYPE_UP })); }
            if to.modifiers.meta && !from.modifiers.meta { block.push_expr(Expr::KeyAction(KeyAction { key: KEY_LEFT_META, value: TYPE_UP })); }

            self.push(Expr::KeyMapping(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_UP, modifiers: from.modifiers.clone() }, to: block }))
        }

        {
            let mut block = Block::new();
            block.push_expr(Expr::KeyAction(KeyAction { key: to.key, value: TYPE_REPEAT }));

            self.push(Expr::KeyMapping(KeyMapping { from: KeyActionWithMods { key: from.key, value: TYPE_REPEAT, modifiers: from.modifiers }, to: block }))
        }
    }

    fn map_key_block(&mut self, from: KeyClickActionWithMods, to: Block) {
        self.push(Expr::KeyMapping(KeyMapping {
            from: KeyActionWithMods {
                key: from.key,
                value: TYPE_DOWN,
                modifiers: from.modifiers,
            },
            to,
        }));
        self.push(Expr::KeyMapping(KeyMapping {
            from: KeyActionWithMods {
                key: from.key,
                value: TYPE_UP,
                modifiers: from.modifiers,
            },
            to: Block::new(),
        }));
        self.push(Expr::KeyMapping(KeyMapping {
            from: KeyActionWithMods {
                key: from.key,
                value: TYPE_REPEAT,
                modifiers: from.modifiers,
            },
            to: Block::new(),
        }));
    }
}


