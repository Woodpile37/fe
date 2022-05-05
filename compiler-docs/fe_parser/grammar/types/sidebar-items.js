initSidebarItems({"fn":[["parse_event_def","Parse an event definition."],["parse_event_field","Parse an event field, e.g. `foo: u8` or `idx from: address`."],["parse_field","Parse a field for a struct or contract. The leading optional `pub` and `const` qualifiers must be parsed by the caller, and passed in. Note that `event` fields are handled in [`parse_event_field`]."],["parse_generic_args","Parse an angle-bracket-wrapped list of generic arguments (eg. the tail end of `Map<address, u256>`)."],["parse_opt_qualifier","Parse an optional qualifier (`pub`, `const`, or `idx`)."],["parse_path_tail","Returns path and trailing `::` token, if present."],["parse_struct_def","Parse a [`ModuleStmt::Struct`]."],["parse_type_alias","Parse a type alias definition, e.g. `type MyMap = Map<u8, address>`."],["parse_type_desc","Parse a type description, e.g. `u8` or `Map<address, u256>`."]]});