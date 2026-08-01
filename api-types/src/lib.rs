macro_rules! api_type {
    (
        $(#[$ty_attrs:meta])*
        struct $name:ident {$(
            $(#[$field_attr:meta])*
            $field_name:ident: $field_ty:ty
        ),* $(,)?}
    ) => {
        $(#[$ty_attrs])*
        #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        pub struct $name {$(
            $(#[$field_attr])*
            pub $field_name: $field_ty,
        )*}
    };

    (
        $(#[$ty_attrs:meta])*
        enum $name:ident {$(
            $(#[$variant_attr:meta])*
            $variant_name:ident$(($(#[$inner_attr:meta])*$inner:ty))?
        ),* $(,)?}
    ) => {
        $(#[$ty_attrs])*
        #[derive(Clone, Debug, PartialEq, ::serde::Deserialize, ::serde::Serialize)]
        pub enum $name {$(
            $(#[$field_attr])*
            $variant_name$($(#[$inner_attr])* $inner)?
        )*}
    };
}

#[derive(Clone, Debug, PartialEq, serde::Deserialize, ::serde::Serialize)]
pub enum ApiResponse<V=(), E=()> {
    Ok(V),
    ServerError(ServerError),
    ClientError(E),
}
api_type!(struct ServerError {
    id: String,
    message: String,
});

