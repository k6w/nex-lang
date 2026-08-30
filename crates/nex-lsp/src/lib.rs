use std::sync::Arc;
use tokio::sync::Mutex;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<std::collections::HashMap<Url, String>>>,
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                document_formatting_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "NEX language server initialized!")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.validate_document(params.text_document.uri, params.text_document.text)
            .await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let text = params.content_changes.into_iter().next().unwrap().text;
        self.validate_document(params.text_document.uri, text).await;
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            match nex_parser::parse(text) {
                Ok(parsed) => {
                    let formatted = nex_formatter::format(&parsed);
                    if formatted != *text {
                        let edit = TextEdit {
                            range: Range {
                                start: Position {
                                    line: 0,
                                    character: 0,
                                },
                                end: Position {
                                    line: text.lines().count() as u32,
                                    character: text.lines().last().map(|l| l.len()).unwrap_or(0)
                                        as u32,
                                },
                            },
                            new_text: formatted,
                        };
                        Ok(Some(vec![edit]))
                    } else {
                        Ok(Some(vec![]))
                    }
                }
                Err(_) => Ok(None),
            }
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            if let Ok(parsed) = nex_parser::parse(text) {
                let symbols = extract_symbols(&parsed, text);
                Ok(Some(DocumentSymbolResponse::Nested(symbols)))
            } else {
                Ok(Some(DocumentSymbolResponse::Nested(vec![])))
            }
        } else {
            Ok(Some(DocumentSymbolResponse::Nested(vec![])))
        }
    }

    async fn hover(&self, _params: HoverParams) -> Result<Option<Hover>> {
        // Basic hover - could show type info
        Ok(None)
    }
}

impl Backend {
    async fn validate_document(&self, uri: Url, text: String) {
        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), text.clone());
        }
        // Check if parsing succeeds and provide diagnostics
        match nex_parser::parse(&text) {
            Ok(_) => {
                // Clear diagnostics
                self.client.publish_diagnostics(uri, vec![], None).await;
            }
            Err(err) => {
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position {
                            line: err.line as u32 - 1,
                            character: err.column as u32 - 1,
                        },
                        end: Position {
                            line: err.line as u32 - 1,
                            character: err.column as u32,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("nex".to_string()),
                    message: err.message,
                    related_information: None,
                    tags: None,
                    data: None,
                };

                self.client
                    .publish_diagnostics(uri, vec![diagnostic], None)
                    .await;
            }
        }
    }
}

fn extract_symbols(value: &nex_parser::ast::Value, text: &str) -> Vec<DocumentSymbol> {
    let mut symbols = Vec::new();

    match value {
        nex_parser::ast::Value::Object { name, fields } => {
            if let Some(name) = name {
                // Find the position of the object
                if let Some(start_pos) = text.find(name) {
                    let start_line = text[..start_pos].lines().count() as u32 - 1;
                    let start_char = text[..start_pos].lines().last().unwrap_or("").len() as u32;

                    let end_pos = start_pos + name.len() + 1; // +1 for {
                    let end_line = text[..end_pos].lines().count() as u32 - 1;
                    let end_char = text[..end_pos].lines().last().unwrap_or("").len() as u32;

                    let mut children = Vec::new();
                    for (field_name, _) in fields {
                        // Find field positions
                        if let Some(field_pos) = text[start_pos..].find(field_name) {
                            let field_start = start_pos + field_pos;
                            let field_line = text[..field_start].lines().count() as u32 - 1;
                            let field_char =
                                text[..field_start].lines().last().unwrap_or("").len() as u32;

                            children.push(DocumentSymbol {
                                name: field_name.clone(),
                                detail: None,
                                kind: SymbolKind::PROPERTY,
                                range: Range {
                                    start: Position {
                                        line: field_line,
                                        character: field_char,
                                    },
                                    end: Position {
                                        line: field_line,
                                        character: field_char + field_name.len() as u32,
                                    },
                                },
                                selection_range: Range {
                                    start: Position {
                                        line: field_line,
                                        character: field_char,
                                    },
                                    end: Position {
                                        line: field_line,
                                        character: field_char + field_name.len() as u32,
                                    },
                                },
                                children: None,
                                tags: None,
                                deprecated: None,
                            });
                        }
                    }

                    symbols.push(DocumentSymbol {
                        name: name.clone(),
                        detail: Some("object".to_string()),
                        kind: SymbolKind::OBJECT,
                        range: Range {
                            start: Position {
                                line: start_line,
                                character: start_char,
                            },
                            end: Position {
                                line: end_line,
                                character: end_char,
                            },
                        },
                        selection_range: Range {
                            start: Position {
                                line: start_line,
                                character: start_char,
                            },
                            end: Position {
                                line: start_line,
                                character: start_char + name.len() as u32,
                            },
                        },
                        children: Some(children),
                        tags: None,
                        deprecated: None,
                    });
                }
            }
        }
        _ => {}
    }

    symbols
}

pub async fn run_lsp() -> Result<()> {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend {
        client,
        documents: Arc::new(Mutex::new(std::collections::HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;

    Ok(())
}
