use crate::*;

impl Block {
    pub(crate) fn append_stmt(mut self, stmt: Stmt) -> Self {
        self.statements.push(stmt);
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

    pub(crate) fn apply_expr(&mut self, expr_vec: Vec<Expr>) {
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
    }

    pub(crate) fn append_string_sequence(mut self, sequence: String) -> Self {
        let expr_vec = vec![].append_string_sequence(sequence);
        self.statements.extend(expr_vec.into_iter().map(|expr| Stmt::Expr(expr)));
        self
    }

    pub(crate) fn sleep_for(mut self, duration: time::Duration) -> Self {
        self.statements.push(Stmt::Expr(Expr::SleepAction(duration)));
        self
    }
}

pub(crate) trait ExprExt {
    fn append_click(self, key: Key) -> Self;
    fn append_action(self, action: KeyAction) -> Self;
    fn sleep_for_millis(self, duration: u64) -> Self;
    fn append_string_sequence(self, sequence: String) -> Self;
}

impl ExprExt for Vec<Expr> {
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

    fn append_string_sequence(mut self, sequence: String) -> Self {
        let mut it = sequence.chars();

        while let Some(ch) = it.next() {
            // special
            if ch == '{' {
                let special_char = it.by_ref().take_while(|&ch| ch != '}').collect::<String>();
                let seq = KEY_LOOKUP.get(special_char.as_str())
                    .expect(format!("failed to lookup key '{}'", special_char).as_str());
                self.extend(seq.iter().cloned());
                continue;
            }

            let seq = KEY_LOOKUP.get(ch.to_string().as_str())
                .expect(format!("failed to lookup key '{}'", ch).as_str());
            self.extend(seq.iter().cloned());
        }
        self
    }
}