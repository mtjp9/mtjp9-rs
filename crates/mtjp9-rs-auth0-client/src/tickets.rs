//! Auth0 Tickets API module

pub mod post_password_change;

pub use post_password_change::{
    create_password_change_ticket, CreatePasswordChangeTicketRequest,
    CreatePasswordChangeTicketRequestBuilder, CreatePasswordChangeTicketResponse,
};