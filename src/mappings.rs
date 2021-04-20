use crate::*;


pub(crate) fn bind_mappings() -> Block {
    let mut global = Block::new();

    global.replace_key(
        KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() },
        KeyAction::new(KEY_A, TYPE_DOWN),
    );

    global.replace_key(
        KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
        KeyAction::new(KEY_A, TYPE_UP),
    );

    global.replace_key_block(
        KeyActionWithMods { key: KEY_MOUSE6, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() },
        Block::new()
            .append_stmt(Stmt::If(Expr::Boolean(false),
                                  Block::new()
                                      .append_string_sequence("a".to_string()),
            ))
            // .sleep_for(time::Duration::from_millis(1000))
            .append_string_sequence("b".to_string()),
    );

    global

    // let mut global_mappings = KeyMappings::new();


    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE5), KeyClickAction::new(KEY_KPD0));

    // let mut seq = KeySequence::new();
    // seq.0.push(KeySequenceItem::Assignment("foo".to_string(), ValueType::Bool(true)));
    // seq = seq.append_string_sequence("hello".to_string());
    //
    // global_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    // global_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    // global_mappings.replace_key(
    //     KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //     seq
    //     ,
    // );
    //
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE6), KeyClickAction::new(KEY_KPD1));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE7), KeyClickAction::new(KEY_KPD2));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE8), KeyClickAction::new(KEY_KPD3));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE9), KeyClickAction::new(KEY_KPD4));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE10), KeyClickAction::new(KEY_KPD5));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE11), KeyClickAction::new(KEY_KPD6));
    // global_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE12), KeyClickAction::new(KEY_KPD7));
    //
    //
    // { // figma-linux
    //     let mut local_mappings = KeyMappings::new();
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE5, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}palette-pick{enter}".to_string()),
    //     );
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE6, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE6, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE6, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}atom-sync{enter}".to_string()),
    //     );
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE7, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE7, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE7, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}batch styler{enter}".to_string()),
    //     );
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE8, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE8, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE8, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}chroma colors{enter}".to_string()),
    //     );
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE9, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE9, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE9, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}scripter{enter}".to_string()),
    //     );
    //
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE11, value: TYPE_DOWN, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(KeyActionWithMods { key: KEY_MOUSE11, value: TYPE_REPEAT, modifiers: KeyModifierFlags::new() }, KeySequence::new());
    //     local_mappings.replace_key(
    //         KeyActionWithMods { key: KEY_MOUSE11, value: TYPE_UP, modifiers: KeyModifierFlags::new() },
    //         KeySequence::new().append_string_sequence("{ctrl down}/{ctrl up}theme-flip{enter}".to_string()),
    //     );
    //
    //     let mut scope = Scope::new();
    //     scope.instructions = vec![ScopeInstruction::KeyMapping(local_mappings)];
    //     scope.condition = Some(KeyActionCondition { window_class_name: Some("figma-linux".to_string()) });
    //
    //     global_scope.push_scope(scope);
    // }
    //
    // { // arrow keys
    //     global_mappings.replace_key_click(
    //         KeyClickAction::new_mods(KEY_H, *KeyModifierFlags::new().alt()),
    //         KeyClickAction::new(KEY_LEFT),
    //     );
    //     global_mappings.replace_key_click(
    //         KeyClickAction::new_mods(KEY_L, *KeyModifierFlags::new().alt()),
    //         KeyClickAction::new(KEY_RIGHT),
    //     );
    //     global_mappings.replace_key_click(
    //         KeyClickAction::new_mods(KEY_K, *KeyModifierFlags::new().alt()),
    //         KeyClickAction::new(KEY_UP),
    //     );
    //     global_mappings.replace_key_click(
    //         KeyClickAction::new_mods(KEY_J, *KeyModifierFlags::new().alt()),
    //         KeyClickAction::new(KEY_DOWN),
    //     );
    // }
    //
    // global_scope.instructions.push(ScopeInstruction::KeyMapping(global_mappings));
    //
    //
    // { // firefox
    //     let mut local_mappings = KeyMappings::new();
    //
    //     local_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE5), KeyClickAction::new_mods(KEY_TAB, *KeyModifierFlags::new().ctrl()));
    //     local_mappings.replace_key_click(KeyClickAction::new_mods(KEY_MOUSE5, *KeyModifierFlags::new().shift()), KeyClickAction::new_mods(KEY_TAB, *KeyModifierFlags::new().ctrl().shift()));
    //     local_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE6), KeyClickAction::new_mods(KEY_T, *KeyModifierFlags::new().ctrl()));
    //     local_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE7), KeyClickAction::new(KEY_F5));
    //     local_mappings.replace_key_click(KeyClickAction::new(KEY_MOUSE12), KeyClickAction::new_mods(KEY_W, *KeyModifierFlags::new().ctrl()));
    //
    //     let mut scope = Scope::new();
    //     scope.instructions = vec![ScopeInstruction::KeyMapping(local_mappings)];
    //     scope.condition = Some(KeyActionCondition { window_class_name: Some("firefox".to_string()) });
    //
    //     global_scope.push_scope(scope);
    //
    //     // global_scope.instructions.push(ScopeInstruction::Scope(Scope {
    //     //     condition: Some(KeyActionCondition { window_class_name: Some("firefox".to_string()) }),
    //     //     instructions: vec![ScopeInstruction::KeyMapping(local_mappings)],
    //     // }));
    // }
    //
    // global_scope
}
