mod ext;
extern crate proc_macro;
use proc_macro::{token_stream::IntoIter, Group, TokenStream, TokenTree};
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use std::collections::HashMap;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Expr, ExprLit, Fields, Lit, Meta, MetaNameValue,
    Variant,
};

/// A procedural macro that derives a `Description` trait for enums.
/// This macro extracts documentation comments (specified with `/// Description...`) for enum variants
/// and generates an implementation for `get_description`, which returns the associated description.
#[proc_macro_derive(Description, attributes(desc))]
pub fn derive_description_from_doc(item: TokenStream) -> TokenStream {
    // Convert the TokenStream into an iterator of TokenTree
    let mut tokens = item.into_iter();

    let mut enum_name = String::new();

    // Vector to store enum variants and their associated payloads (if any)
    let mut enum_variants: Vec<(String, Option<String>)> = Vec::<(String, Option<String>)>::new();

    // HashMap to store descriptions associated with each enum variant
    let mut variant_description_map: HashMap<String, String> = HashMap::new();

    // Parses the token stream to extract the enum name and its variants
    while let Some(token) = tokens.next() {
        match token {
            TokenTree::Ident(ident) if ident.to_string() == "enum" => {
                // Get the enum name
                if let Some(TokenTree::Ident(name)) = tokens.next() {
                    enum_name = name.to_string();
                }
            }
            TokenTree::Group(group) => {
                let mut group_tokens_iter: IntoIter = group.stream().into_iter();

                let mut last_seen_desc: Option<String> = None;
                while let Some(token) = group_tokens_iter.next() {
                    match token {
                        TokenTree::Punct(punct) => {
                            if punct.to_string() == "#" {
                                last_seen_desc = process_description(&mut group_tokens_iter);
                            }
                        }
                        TokenTree::Ident(ident) => {
                            // Capture the enum variant name and associate it with its description
                            let ident_str = ident.to_string();
                            if let Some(desc) = &last_seen_desc {
                                variant_description_map.insert(ident_str.clone(), desc.clone());
                            }
                            enum_variants.push((ident_str, None));
                            last_seen_desc = None;
                        }
                        TokenTree::Group(group) => {
                            // Capture payload information for the current enum variant
                            if let Some(last_variant) = enum_variants.last_mut() {
                                last_variant.1 = Some(process_payload(group));
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }
    generate_get_description(enum_name, &variant_description_map, enum_variants)
}

/// Processes a Rust docs to extract the description string.
fn process_description(token_iter: &mut IntoIter) -> Option<String> {
    if let Some(TokenTree::Group(doc_group)) = token_iter.next() {
        let mut doc_group_iter = doc_group.stream().into_iter();
        // Skip the `desc` and `(` tokens to reach the actual description
        doc_group_iter.next();
        doc_group_iter.next();
        if let Some(TokenTree::Literal(description)) = doc_group_iter.next() {
            return Some(description.to_string());
        }
    }
    None
}

/// Processes the payload of an enum variant to extract variable names (ignoring types).
fn process_payload(payload_group: Group) -> String {
    let payload_group_iter = payload_group.stream().into_iter();
    let mut variable_name_list = String::from("");
    let mut is_variable_name = true;
    for token in payload_group_iter {
        match token {
            TokenTree::Ident(ident) => {
                if is_variable_name {
                    variable_name_list.push_str(&format!("{ident},"));
                }
                is_variable_name = false;
            }
            TokenTree::Punct(punct) => {
                if punct.to_string() == "," {
                    is_variable_name = true;
                }
            }
            _ => {}
        }
    }
    format!("{{ {variable_name_list} }}").to_string()
}
/// Generates the `get_description` implementation for the processed enum.
fn generate_get_description(
    enum_name: String,
    variant_description_map: &HashMap<String, String>,
    enum_variants: Vec<(String, Option<String>)>,
) -> TokenStream {
    let mut all_enum_arms = String::from("");
    for (variant, payload) in enum_variants {
        let payload = payload.unwrap_or("".to_string());
        let desc;
        if let Some(description) = variant_description_map.get(&variant) {
            desc = format!("Some({description})");
        } else {
            desc = "None".to_string();
        }
        all_enum_arms.push_str(&format!("{enum_name}::{variant} {payload} => {desc},\n"));
    }

    let enum_impl = format!(
        "impl {enum_name}  {{
     pub fn get_description(&self) -> Option<&str> {{
     match self {{
     {all_enum_arms}
     }}
     }}
     }}"
    );
    enum_impl.parse().unwrap()
}

/// Register your extension with 'core' by providing the relevant functions
///```ignore
///use turso_ext::{register_extension, scalar, Value, AggregateDerive, AggFunc};
///
/// register_extension!{ scalars: { return_one }, aggregates: { SumPlusOne } }
///
///#[scalar(name = "one")]
///fn return_one(args: &[Value]) -> Value {
///  return Value::from_integer(1);
///}
///
///#[derive(AggregateDerive)]
///struct SumPlusOne;
///
///impl AggFunc for SumPlusOne {
///   type State = i64;
///   const NAME: &'static str = "sum_plus_one";
///   const ARGS: i32 = 1;
///
///   fn step(state: &mut Self::State, args: &[Value]) {
///      let Some(val) = args[0].to_integer() else {
///        return;
///      };
///      *state += val;
///     }
///
///     fn finalize(state: Self::State) -> Value {
///        Value::from_integer(state + 1)
///     }
///}
///
/// ```
#[proc_macro]
pub fn register_extension(input: TokenStream) -> TokenStream {
    ext::register_extension(input)
}

/// Declare a scalar function for your extension. This requires the name:
/// #[scalar(name = "example")] of what you wish to call your function with.
/// ```ignore
/// use turso_ext::{scalar, Value};
/// #[scalar(name = "double", alias = "twice")] // you can provide an <optional> alias
/// fn double(args: &[Value]) -> Value {
///       let arg = args.get(0).unwrap();
///       match arg.value_type() {
///           ValueType::Float => {
///               let val = arg.to_float().unwrap();
///               Value::from_float(val * 2.0)
///           }
///           ValueType::Integer => {
///               let val = arg.to_integer().unwrap();
///               Value::from_integer(val * 2)
///           }
///       }
///   } else {
///       Value::null()
///   }
/// }
/// ```
#[proc_macro_attribute]
pub fn scalar(attr: TokenStream, input: TokenStream) -> TokenStream {
    ext::scalar(attr, input)
}

/// Define an aggregate function for your extension by deriving
/// AggregateDerive on a struct that implements the AggFunc trait.
/// ```ignore
/// use turso_ext::{register_extension, Value, AggregateDerive, AggFunc};
///
///#[derive(AggregateDerive)]
///struct SumPlusOne;
///
///impl AggFunc for SumPlusOne {
///   type State = i64;
///   type Error = &'static str;
///   const NAME: &'static str = "sum_plus_one";
///   const ARGS: i32 = 1;
///   fn step(state: &mut Self::State, args: &[Value]) {
///      let Some(val) = args[0].to_integer() else {
///        return;
///     };
///     *state += val;
///     }
///     fn finalize(state: Self::State) -> Result<Value, Self::Error> {
///        Ok(Value::from_integer(state + 1))
///     }
///}
/// ```
#[proc_macro_derive(AggregateDerive)]
pub fn derive_agg_func(input: TokenStream) -> TokenStream {
    ext::derive_agg_func(input)
}

/// Macro to derive a VTabModule for your extension. This macro will generate
/// the necessary functions to register your module with core. You must implement
/// the VTabModule, VTable, and VTabCursor traits.
/// ```ignore
/// #[derive(Debug, VTabModuleDerive)]
/// struct CsvVTabModule;
///
/// impl VTabModule for CsvVTabModule {
///  type Table = CsvTable;
///  const NAME: &'static str = "csv_data";
///  const VTAB_KIND: VTabKind = VTabKind::VirtualTable;
///
///   /// Declare your virtual table and its schema
///  fn create(args: &[Value]) -> Result<(String, Self::Table), ResultCode> {
///     let schema = "CREATE TABLE csv_data(
///             name TEXT,
///             age TEXT,
///             city TEXT
///         )".into();
///     Ok((schema, CsvTable {}))
///  }
/// }
///
/// struct CsvTable {}
///
/// // Implement the VTable trait for your virtual table
/// impl VTable for CsvTable {
///  type Cursor = CsvCursor;
///  type Error = &'static str;
///
///  /// Open the virtual table and return a cursor
///  fn open(&self) -> Result<Self::Cursor, Self::Error> {
///     let csv_content = fs::read_to_string("data.csv").unwrap_or_default();
///     let rows: Vec<Vec<String>> = csv_content
///         .lines()
///         .skip(1)
///         .map(|line| {
///             line.split(',')
///                 .map(|s| s.trim().to_string())
///                 .collect()
///         })
///         .collect();
///     Ok(CsvCursor { rows, index: 0 })
///  }
///
/// /// **Optional** methods for non-readonly tables:
///
///  /// Update the row with the provided values, return the new rowid
///  fn update(&mut self, rowid: i64, args: &[Value]) -> Result<Option<i64>, Self::Error> {
///      Ok(None)// return Ok(None) for read-only
///  }
///
///  /// Insert a new row with the provided values, return the new rowid
///  fn insert(&mut self, args: &[Value]) -> Result<(), Self::Error> {
///      Ok(()) //
///  }
///
///  /// Delete the row with the provided rowid
///  fn delete(&mut self, rowid: i64) -> Result<(), Self::Error> {
///    Ok(())
///  }
///
///  /// Destroy the virtual table. Any cleanup logic for when the table is deleted comes heres
///  fn destroy(&mut self) -> Result<(), Self::Error> {
///     Ok(())
///  }
/// }
///
///  #[derive(Debug)]
/// struct CsvCursor {
///   rows: Vec<Vec<String>>,
///   index: usize,
/// }
///
/// impl CsvCursor {
///   /// Returns the value for a given column index.
///   fn column(&self, idx: u32) -> Result<Value, Self::Error> {
///       let row = &self.rows[self.index];
///       if (idx as usize) < row.len() {
///           Value::from_text(&row[idx as usize])
///       } else {
///           Value::null()
///       }
///   }
/// }
///
/// // Implement the VTabCursor trait for your virtual cursor
/// impl VTabCursor for CsvCursor {
///  type Error = &'static str;
///
///  /// Filter the virtual table based on arguments (omitted here for simplicity)
///  fn filter(&mut self, _args: &[Value], _idx_info: Option<(&str, i32)>) -> ResultCode {
///      ResultCode::OK
///  }
///
///  /// Move the cursor to the next row
///  fn next(&mut self) -> ResultCode {
///     if self.index < self.rows.len() - 1 {
///         self.index += 1;
///         ResultCode::OK
///     } else {
///         ResultCode::EOF
///     }
///  }
///
///  fn eof(&self) -> bool {
///      self.index >= self.rows.len()
///  }
///
///  /// Return the value for a given column index
///  fn column(&self, idx: u32) -> Result<Value, Self::Error> {
///      self.column(idx)
///  }
///
///  fn rowid(&self) -> i64 {
///      self.index as i64
///  }
/// }
///
#[proc_macro_derive(VTabModuleDerive)]
pub fn derive_vtab_module(input: TokenStream) -> TokenStream {
    ext::derive_vtab_module(input)
}

/// ```ignore
/// use turso_ext::{ExtResult as Result, VfsDerive, VfsExtension, VfsFile};
///
/// // Your struct must also impl Default
/// #[derive(VfsDerive, Default)]
/// struct ExampleFS;
///
///
/// struct ExampleFile {
///    file: std::fs::File,
///
///
/// impl VfsExtension for ExampleFS {
///    /// The name of your vfs module
///    const NAME: &'static str = "example";
///
///    type File = ExampleFile;
///
///    fn open(&self, path: &str, flags: i32, _direct: bool) -> Result<Self::File> {
///        let file = OpenOptions::new()
///            .read(true)
///            .write(true)
///            .create(flags & 1 != 0)
///            .open(path)
///            .map_err(|_| ResultCode::Error)?;
///        Ok(TestFile { file })
///    }
///
///    fn run_once(&self) -> Result<()> {
///    // (optional) method to cycle/advance IO, if your extension is asynchronous
///        Ok(())
///    }
///
///    fn close(&self, file: Self::File) -> Result<()> {
///    // (optional) method to close or drop the file
///        Ok(())
///    }
///
///    fn generate_random_number(&self) -> i64 {
///    // (optional) method to generate random number. Used for testing
///        let mut buf = [0u8; 8];
///        getrandom::fill(&mut buf).unwrap();
///        i64::from_ne_bytes(buf)
///    }
///
///   fn get_current_time(&self) -> String {
///    // (optional) method to generate random number. Used for testing
///        chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
///    }
///
///
/// impl VfsFile for ExampleFile {
///    fn read(
///        &mut self,
///        buf: &mut [u8],
///        count: usize,
///        offset: i64,
///    ) -> Result<i32> {
///        if file.file.seek(SeekFrom::Start(offset as u64)).is_err() {
///            return Err(ResultCode::Error);
///        }
///        file.file
///            .read(&mut buf[..count])
///            .map_err(|_| ResultCode::Error)
///            .map(|n| n as i32)
///    }
///
///    fn write(&mut self, buf: &[u8], count: usize, offset: i64) -> Result<i32> {
///        if self.file.seek(SeekFrom::Start(offset as u64)).is_err() {
///            return Err(ResultCode::Error);
///        }
///        self.file
///            .write(&buf[..count])
///            .map_err(|_| ResultCode::Error)
///            .map(|n| n as i32)
///    }
///
///    fn sync(&self) -> Result<()> {
///        self.file.sync_all().map_err(|_| ResultCode::Error)
///    }
///
///    fn size(&self) -> i64 {
///      self.file.metadata().map(|m| m.len() as i64).unwrap_or(-1)
///   }
///}
///
///```
#[proc_macro_derive(VfsDerive)]
pub fn derive_vfs_module(input: TokenStream) -> TokenStream {
    ext::derive_vfs_module(input)
}

/// Derives opcode functionality for database instruction enums.
///
/// This macro generates three methods:
/// - `name()`: Returns the instruction name as a string
/// - `description()`: Returns the instruction description
/// - `explain()`: Returns detailed instruction information for debugging
///
/// It also generates an `OPCODE_DICTIONARY` constant containing all opcodes.
///
/// # Required Attributes
///
/// Each enum variant must have:
/// - `#[description = "..."]` - Human readable description of what the instruction does
///
/// Each field that represents an opcode parameter must have one of:
/// - `#[p1]` through `#[p5]` - Parameter position markers (up to 5 parameters supported)
///
/// # Optional Attributes
///
/// - `#[format = "..."]` - Custom format string for explain() method. Use `{p1}`, `{p2}`, etc.
///   as placeholders for parameters. Defaults to a generic format if not specified.
///
/// # Example
///
/// ```rust
/// #[derive(Opcode)]
/// pub enum Insn {
///     #[description = "Jump to P2 if r[P1] is true, or if r[P1] is null and r[P3] is true"]
///     #[format = "if r[{p1}] goto {p2} else if null(r[{p1}]) and r[{p3}] goto {p2}"]
///     If {
///         #[p1] reg: usize,
///         #[p2] target_pc: BranchOffset,
///         #[p3] jump_if_null: bool,
///     },
///
///     #[description = "Unconditional jump to target address"]
///     #[format = "goto {p1}"]
///     Goto {
///         #[p1] target_pc: BranchOffset,
///     },
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates:
/// - `name(&self) -> &'static str` - Returns variant name
/// - `description(&self) -> &'static str` - Returns description from attribute
/// - `explain(&self) -> InstructionExplanation` - Returns detailed explanation
/// - `OPCODE_DICTIONARY: &[OpCodeDescription]` - Array of all opcodes and descriptions
///
/// # Errors
///
/// Compilation will fail if:
/// - Applied to non-enum types
/// - Any variant is missing `#[description = "..."]` attribute
/// - Parameter attributes `#[p1]` through `#[p5]` are used incorrectly
/// - The same parameter number is used twice in one variant
/// - Format strings reference non-existent parameters
#[proc_macro_derive(Opcode, attributes(description, format, p1, p2, p3, p4, p5))]
pub fn derive_opcode(input: TokenStream) -> TokenStream {
    derive_opcode_impl(input).unwrap_or_else(|err| err.to_compile_error().into())
}

/// Internal implementation that returns proper syn::Result for error handling
fn derive_opcode_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = syn::parse(input)?;
    let enum_ident = &input.ident;

    let Data::Enum(DataEnum { variants, .. }) = &input.data else {
        return Err(syn::Error::new_spanned(
            enum_ident,
            "Opcode can only be derived for enums",
        ));
    };

    // Validate all variants and collect their information
    let mut variant_info = Vec::new();
    for variant in variants {
        let info = validate_and_extract_variant(variant, enum_ident)?;
        variant_info.push(info);
    }

    let mut arms_name = TokenStream2::new();
    let mut arms_desc = TokenStream2::new();
    let mut arms_exp = TokenStream2::new();
    let mut dict_elems = TokenStream2::new();

    for info in &variant_info {
        expand_one_variant(
            enum_ident,
            info,
            &mut arms_name,
            &mut arms_desc,
            &mut arms_exp,
            &mut dict_elems,
        );
    }

    // Generate unique type names to avoid conflicts
    let explanation_type = format_ident!("{}InstructionExplanation", enum_ident);
    let dictionary_name = format_ident!(
        "{}_OPCODE_DICTIONARY",
        enum_ident.to_string().to_uppercase()
    );

    let expanded = quote! {
        /// Information about an instruction for debugging and explanation
        #[derive(Debug, Clone)]
        pub struct #explanation_type {
            /// The instruction name
            pub name: &'static str,
            /// Parameter 1 value (0 if not used)
            pub p1: i32,
            /// Parameter 2 value (0 if not used)
            pub p2: i32,
            /// Parameter 3 value (0 if not used)
            pub p3: i32,
            /// Parameter 4 value (0 if not used)
            pub p4: i32,
            /// Parameter 5 value (0 if not used)
            pub p5: i32,
            /// Human-readable formatted explanation of what this instruction does
            pub explanation: String,
        }

        impl #enum_ident {
            /// Returns the name of this instruction as a static string
            pub const fn name(&self) -> &'static str {
                match self {
                    #arms_name
                }
            }

            /// Returns the description of this instruction as a static string
            pub const fn description(&self) -> &'static str {
                match self {
                    #arms_desc
                }
            }

            /// Returns detailed explanation of this instruction instance including parameter values
            pub fn explain(&self) -> #explanation_type {
                match self {
                    #arms_exp
                }
            }
        }

        /// Dictionary of all opcodes with their names and descriptions
        pub const #dictionary_name: &[OpCodeDescription] = &[#dict_elems];
    };

    Ok(expanded.into())
}

/// Information extracted from a validated enum variant
#[derive(Debug)]
struct VariantInfo {
    ident: syn::Ident,
    description: String,
    format_string: Option<String>,
    parameters: Vec<ParameterInfo>,
    pattern: TokenStream2,
}

/// Information about a parameter in a variant
#[derive(Debug)]
struct ParameterInfo {
    position: usize, // 1-5 for p1-p5
    binding: syn::Ident,
    field_name: Option<syn::Ident>, // None for tuple fields
}

/// Validates a variant and extracts all necessary information
fn validate_and_extract_variant(
    variant: &Variant,
    enum_ident: &syn::Ident,
) -> syn::Result<VariantInfo> {
    let var_ident = variant.ident.clone();

    // Extract and validate description attribute
    let description = extract_description(&variant.attrs)?;

    // Extract optional format string
    let format_string = extract_format_string(&variant.attrs)?;

    // Extract and validate parameters
    let parameters = extract_parameters(variant)?;

    // Validate format string references existing parameters
    if let Some(ref fmt) = format_string {
        validate_format_string(fmt, &parameters)?;
    }

    // Generate the pattern for matching
    let pattern = generate_pattern(enum_ident, variant, &parameters)?;

    Ok(VariantInfo {
        ident: var_ident,
        description,
        format_string,
        parameters,
        pattern,
    })
}

/// Extracts the description attribute value
fn extract_description(attrs: &[Attribute]) -> syn::Result<String> {
    let desc_attr = find_attr(attrs, "description").ok_or_else(|| {
        syn::Error::new(
            proc_macro2::Span::call_site(),
            "Missing #[description = \"...\"] attribute on opcode variant",
        )
    })?;

    parse_str_lit(desc_attr)
}

/// Extracts the optional format attribute value
fn extract_format_string(attrs: &[Attribute]) -> syn::Result<Option<String>> {
    if let Some(fmt_attr) = find_attr(attrs, "format") {
        Ok(Some(parse_str_lit(fmt_attr)?))
    } else {
        Ok(None)
    }
}

/// Extracts and validates parameter information from variant fields
fn extract_parameters(variant: &Variant) -> syn::Result<Vec<ParameterInfo>> {
    let mut parameters = Vec::new();
    let mut used_positions = std::collections::HashSet::new();

    match &variant.fields {
        Fields::Named(fields) => {
            for field in &fields.named {
                let field_name = field.ident.as_ref().ok_or_else(|| {
                    syn::Error::new_spanned(field, "Named field missing identifier")
                })?;

                if let Some(position) = find_param_position(&field.attrs)? {
                    if !used_positions.insert(position) {
                        return Err(syn::Error::new_spanned(
                            field,
                            format!(
                                "Parameter p{} is used more than once in this variant",
                                position
                            ),
                        ));
                    }

                    let binding = format_ident!("_{}", field_name);
                    parameters.push(ParameterInfo {
                        position,
                        binding,
                        field_name: Some(field_name.clone()),
                    });
                }
            }
        }
        Fields::Unnamed(fields) => {
            for (idx, field) in fields.unnamed.iter().enumerate() {
                if let Some(position) = find_param_position(&field.attrs)? {
                    if !used_positions.insert(position) {
                        return Err(syn::Error::new_spanned(
                            field,
                            format!(
                                "Parameter p{} is used more than once in this variant",
                                position
                            ),
                        ));
                    }

                    let binding = format_ident!("_f{}", idx);
                    parameters.push(ParameterInfo {
                        position,
                        binding,
                        field_name: None,
                    });
                }
            }
        }
        Fields::Unit => {
            // Unit variants have no parameters, which is fine
        }
    }

    // Sort by position for consistent ordering
    parameters.sort_by_key(|p| p.position);

    Ok(parameters)
}

fn find_param_position(attrs: &[Attribute]) -> syn::Result<Option<usize>> {
    for attr in attrs {
        if let Some(ident) = attr.path().get_ident() {
            if let Some(n_str) = ident.to_string().strip_prefix('p') {
                if let Ok(n) = n_str.parse::<usize>() {
                    if (1..=5).contains(&n) {
                        if !matches!(attr.meta, Meta::Path(_)) {
                            return Err(syn::Error::new_spanned(
                                attr,
                                format!("Attribute #[p{}] should not have a value", n),
                            ));
                        }
                        return Ok(Some(n));
                    }
                }
            }
        }
    }

    Ok(None)
}

/// Validates that format string only references existing parameters
fn validate_format_string(format_str: &str, parameters: &[ParameterInfo]) -> syn::Result<()> {
    let available_params: std::collections::HashSet<usize> =
        parameters.iter().map(|p| p.position).collect();

    // Simple validation - look for {p1}, {p2}, etc. in format string
    for n in 1..=5 {
        let param_ref = format!("{{p{}}}", n);
        if format_str.contains(&param_ref) && !available_params.contains(&n) {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                format!(
                    "Format string references {{p{}}} but no field has #[p{}] attribute",
                    n, n
                ),
            ));
        }
    }

    Ok(())
}

/// Generates the pattern for matching this variant
fn generate_pattern(
    enum_ident: &syn::Ident,
    variant: &Variant,
    parameters: &[ParameterInfo],
) -> syn::Result<TokenStream2> {
    let var_ident = &variant.ident;

    let bindings = match &variant.fields {
        Fields::Named(_) => {
            let mut inner = TokenStream2::new();

            // Add bindings for parameter fields
            for param in parameters {
                if let Some(ref field_name) = param.field_name {
                    let binding = &param.binding;
                    inner.extend(quote! { #field_name: #binding, });
                }
            }

            // Add bindings for non-parameter fields (using underscore to ignore)
            if let Fields::Named(fields) = &variant.fields {
                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    // Only add if it's not already a parameter
                    if !parameters
                        .iter()
                        .any(|p| p.field_name.as_ref() == Some(field_name))
                    {
                        inner.extend(quote! { #field_name: _, });
                    }
                }
            }

            quote! { { #inner } }
        }
        Fields::Unnamed(fields) => {
            let mut inner = TokenStream2::new();

            for (idx, _) in fields.unnamed.iter().enumerate() {
                // Check if this field is a parameter
                if let Some(param) = parameters
                    .iter()
                    .find(|p| p.field_name.is_none() && p.binding == format_ident!("_f{}", idx))
                {
                    let binding = &param.binding;
                    inner.extend(quote! { #binding, });
                } else {
                    inner.extend(quote! { _, });
                }
            }

            quote! { ( #inner ) }
        }
        Fields::Unit => quote! {},
    };

    Ok(quote! { #enum_ident::#var_ident #bindings })
}

/// Expands a single variant into the generated match arms
fn expand_one_variant(
    _enum_ident: &syn::Ident,
    info: &VariantInfo,
    arms_name: &mut TokenStream2,
    arms_desc: &mut TokenStream2,
    arms_exp: &mut TokenStream2,
    dict_elems: &mut TokenStream2,
) {
    let var_name = info.ident.to_string();
    let description = &info.description;
    let pattern = &info.pattern;

    // Generate name match arm
    arms_name.extend(quote! { #pattern => #var_name, });

    // Generate description match arm
    arms_desc.extend(quote! { #pattern => #description, });

    // Generate parameter values array (p1-p5)
    let mut param_values = vec![quote! { 0i32 }; 5];
    for param in &info.parameters {
        let binding = &param.binding;
        param_values[param.position - 1] = quote! { (*#binding) as i32 };
    }

    // Generate explanation string
    let explanation = if let Some(ref format_str) = info.format_string {
        // Use custom format string, replacing {p1}, {p2}, etc. with actual values
        let mut format_parts = Vec::new();
        let mut format_args = Vec::new();
        let mut current_str = format_str.as_str();

        while let Some(start) = current_str.find('{') {
            if let Some(end) = current_str[start..].find('}') {
                let end = start + end;

                // Add the text before the placeholder
                if start > 0 {
                    let before = &current_str[..start];
                    format_parts.push(before.to_string());
                }

                // Handle the placeholder
                let placeholder = &current_str[start + 1..end];
                if let Some(param_num) = placeholder
                    .strip_prefix('p')
                    .and_then(|s| s.parse::<usize>().ok())
                {
                    if param_num >= 1 && param_num <= 5 {
                        if let Some(param) =
                            info.parameters.iter().find(|p| p.position == param_num)
                        {
                            let binding = &param.binding;
                            format_parts.push("{}".to_string());
                            format_args.push(quote! { #binding });
                        } else {
                            format_parts.push("0".to_string());
                        }
                    } else {
                        format_parts.push(format!("{{{}}}", placeholder));
                    }
                } else {
                    format_parts.push(format!("{{{}}}", placeholder));
                }

                current_str = &current_str[end + 1..];
            } else {
                break;
            }
        }

        // Add any remaining text
        if !current_str.is_empty() {
            format_parts.push(current_str.to_string());
        }

        if format_args.is_empty() {
            // No parameters, just use the string literal
            let combined = format_parts.join("");
            quote! { #combined.to_string() }
        } else {
            // Use format! macro with the arguments
            let format_str = format_parts.join("");
            quote! { format!(#format_str, #(#format_args),*) }
        }
    } else {
        // Default format: just show the instruction name and key parameters
        if info.parameters.is_empty() {
            quote! { #var_name.to_string() }
        } else {
            let first_param = &info.parameters[0].binding;
            if info.parameters.len() == 1 {
                quote! { format!("{} {}", #var_name, #first_param) }
            } else {
                let second_param = &info.parameters[1].binding;
                quote! { format!("{} {} {}", #var_name, #first_param, #second_param) }
            }
        }
    };

    // Generate explain match arm
    let p1 = &param_values[0];
    let p2 = &param_values[1];
    let p3 = &param_values[2];
    let p4 = &param_values[3];
    let p5 = &param_values[4];

    let explanation_type = format_ident!("{}InstructionExplanation", _enum_ident);

    arms_exp.extend(quote! {
        #pattern => #explanation_type {
            name: #var_name,
            p1: #p1,
            p2: #p2,
            p3: #p3,
            p4: #p4,
            p5: #p5,
            explanation: #explanation,
        },
    });

    // Generate dictionary element
    dict_elems.extend(quote! {
        OpCodeDescription { name: #var_name, description: #description },
    });
}

/// Finds an attribute by name
fn find_attr<'a>(attrs: &'a [Attribute], want: &str) -> Option<&'a Attribute> {
    attrs.iter().find(|a| a.path().is_ident(want))
}

/// Parses a string literal from an attribute
fn parse_str_lit(attr: &Attribute) -> syn::Result<String> {
    match &attr.meta {
        Meta::NameValue(MetaNameValue {
            value: Expr::Lit(ExprLit {
                lit: Lit::Str(s), ..
            }),
            ..
        }) => Ok(s.value()),
        _ => Err(syn::Error::new_spanned(
            attr,
            "Attribute must be of the form #[attr = \"string value\"]",
        )),
    }
}
