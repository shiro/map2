use crate::*;

impl Block {
    pub(crate) fn push_expr(&mut self, expr: Expr) -> &mut Self {
        self.statements.push(Stmt::Expr(expr));
        self
    }
}

impl Expr {

    pub(crate) fn map_key_click_block(from: KeyClickActionWithMods, mut to: Block) -> Self {
        to.statements.insert(0, Stmt::Expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP)));
        Expr::KeyMapping(vec![
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_DOWN, from.modifiers), to },
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_REPEAT, from.modifiers), to: Block::new() }, // stub
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_UP, from.modifiers), to: Block::new() }, // stub
        ])
    }

    pub(crate) fn map_key_block(from: KeyActionWithMods, mut to: Block) -> Self {
        to.statements.insert(0, Stmt::Expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), KeyModifierFlags::new(), TYPE_UP)));

        Expr::KeyMapping(vec![KeyMapping { from, to }])
    }

    pub(crate) fn map_key_action_action(from: KeyActionWithMods, to: KeyActionWithMods) -> Self {
        let mut block = Block::new();

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

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

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

        Expr::KeyMapping(vec![KeyMapping { from, to: block }])
    }

    // PROTO
    pub(crate) fn map_key_click_action(from: KeyClickActionWithMods, to: KeyActionWithMods) -> Self {
        let mut block = Block::new();

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

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

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

        Expr::KeyMapping(vec![
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_DOWN, KeyModifierFlags::new()), to: block },
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_REPEAT, KeyModifierFlags::new()), to: Block::new() },
            KeyMapping { from: KeyActionWithMods::new(from.key, TYPE_UP, KeyModifierFlags::new()), to: Block::new() },
        ])
    }

    pub(crate) fn map_key_action_click(from: KeyActionWithMods, to: KeyClickActionWithMods) -> Self {
        let mut block = Block::new();

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

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

        block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

        Expr::KeyMapping(vec![KeyMapping { from, to: block }])
    }

    pub(crate) fn map_key_click(from: &KeyClickActionWithMods, to: &KeyClickActionWithMods) -> Self {
        let mut mappings = vec![];
        {
            let mut block = Block::new();
            block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_UP));

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

            block.push_expr(Expr::ReleaseRestoreModifiers(from.modifiers.clone(), to.modifiers.clone(), TYPE_DOWN));

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
}


