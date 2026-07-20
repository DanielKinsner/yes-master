// Optional free-beta email capture configuration. Downloads are always ungated.
//
// Paste your email-provider form-action URL here (Buttondown / MailerLite / Kit
// all give you one). The form POSTs the email as URL-encoded form data.
// Leave SIGNUP_ENDPOINT as "" to show a graceful "sign-up opening soon" state
// instead of submitting — so the page is shippable before the list exists.
export const SIGNUP_ENDPOINT = "";

// The form field name your provider expects for the email address.
// Buttondown: "email"  |  MailerLite: "fields[email]"  |  Kit/ConvertKit: "email_address"
export const SIGNUP_FIELD = "email";
