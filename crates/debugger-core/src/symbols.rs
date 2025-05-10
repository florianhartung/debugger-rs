use crate::Debugger;

use crate::Result;
use elf::{
    ElfBytes, abi::STT_FUNC, endian::AnyEndian, string_table::StringTable, symbol::SymbolTable,
};

impl Debugger {
    fn parse_symbol_table(&self) -> Result<Option<(SymbolTable<'_, AnyEndian>, StringTable)>> {
        let elf_bytes = ElfBytes::<AnyEndian>::minimal_parse(&self.executable_data)?;
        elf_bytes.symbol_table().map_err(Into::into)
    }

    fn find_symbol_by_name(&self, symbol_name: &str) -> Result<Option<elf::symbol::Symbol>> {
        self.parse_symbol_table().map(|tables| {
            tables.and_then(|(symbol_table, string_table)| {
                symbol_table.iter().find(|x| {
                    // 0 means no symbol name
                    if x.st_name == 0 {
                        return false;
                    }
                    // TODO parse errors here should bubble up instead of being ignored
                    string_table.get(x.st_name as usize).map_err(|_| ()) == Ok(symbol_name)
                })
            })
        })
    }

    pub fn find_symbol_address_by_name(&self, symbol_name: &str) -> Result<Option<u64>> {
        self.find_symbol_by_name(symbol_name)
            .map(|symbol| symbol.map(|symbol| symbol.st_value))
    }

    pub fn list_function_symbols(&self) -> Result<Vec<FunctionSymbol>> {
        self.parse_symbol_table().and_then(|tables| {
            tables.map_or_else(
                || Ok(Vec::new()),
                |(symbol_table, string_table)| {
                    symbol_table
                        .iter()
                        .filter(|symbol| symbol.st_symtype() == STT_FUNC)
                        .map(|symbol| {
                            let name = if symbol.st_name > 0 {
                                Some(string_table.get(symbol.st_name as usize)?)
                            } else {
                                None
                            };
                            Ok(FunctionSymbol {
                                name,
                                offset: symbol.st_value,
                            })
                        })
                        .collect()
                },
            )
        })
    }
}

pub struct FunctionSymbol<'a> {
    pub name: Option<&'a str>,
    pub offset: u64,
}
