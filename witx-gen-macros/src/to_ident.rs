use proc_macro2::{Span, TokenStream};
use std::str::FromStr;
use syn::{parse2, Ident};
use witx::Id;

pub(crate) trait ToIdent {
    fn to_ident(&self) -> Ident;
    fn to_ident_native(&self, prefix: Option<&str>) -> Ident;
    fn to_ident_upper(&self) -> Ident;
}

impl ToIdent for str {
    fn to_ident(&self) -> Ident {
        let mut out_buf;

        let out = match self {
            "2big" => "TooBig", // identifiers must not start with a digit
            _ => {
                let mut upper = true;
                out_buf = String::with_capacity(self.len() + 2);

                for c in self.chars() {
                    if c == '_' {
                        upper = true;
                    } else if upper {
                        let c_upper = c.to_uppercase();
                        out_buf.extend(c_upper);
                        upper = false;
                    } else {
                        out_buf.push(c);
                    }
                }
                &out_buf
            }
        };

        Ident::new(out, Span::call_site())
    }

    fn to_ident_native(&self, prefix: Option<&str>) -> Ident {
        let mut name_buf;

        let mut name = match prefix {
            Some(prefix) => {
                name_buf = format!("{}_{}", prefix, self);
                &name_buf
            }
            None => self,
        };

        if is_keyword(name) {
            name_buf = format!("r#{}", name);
            name = &name_buf;
        }

        let stream = TokenStream::from_str(name).expect("Could not parse identifier");
        parse2(stream).expect(&format!("Could not create identifier"))
    }

    fn to_ident_upper(&self) -> Ident {
        Ident::new(&self.to_uppercase(), Span::call_site())
    }
}

impl ToIdent for Id {
    fn to_ident(&self) -> Ident {
        self.as_str().to_ident()
    }

    fn to_ident_native(&self, prefix: Option<&str>) -> Ident {
        self.as_str().to_ident_native(prefix)
    }

    fn to_ident_upper(&self) -> Ident {
        self.as_str().to_ident_upper()
    }
}

fn is_keyword(id: &str) -> bool {
    id == "type" || id == "in"
}
