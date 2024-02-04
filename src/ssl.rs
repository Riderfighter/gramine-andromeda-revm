use openssl::hash::MessageDigest;
use openssl::pkey;
use openssl::pkey::PKey;
use openssl::stack::Stack;
use openssl::x509::{X509Req, X509ReqBuilder};
use openssl::x509::extension::SubjectAlternativeName;

/// Generates the X509 Request for creating an SSL certificate.
pub(crate) fn create_csr(pkey: &PKey<pkey::Private>) -> Result<X509Req> {
    //
    // the csr builder
    let mut req_bld = X509ReqBuilder::new().expect("X509ReqBuilder");

    let mut x509_name = openssl::x509::X509NameBuilder::new().unwrap();
    x509_name.append_entry_by_text("C", "US").unwrap();
    x509_name.append_entry_by_text("ST", "IL").unwrap();
    x509_name.append_entry_by_text("O", "n/a").unwrap();
    x509_name.append_entry_by_text("CN", "*.k37713.xyz").unwrap();
    let x509_name = x509_name.build();

    req_bld.set_subject_name(&x509_name).unwrap();


    // set private/public key in builder
    req_bld.set_pubkey(pkey).expect("set_pubkey");

    // set all domains as alt names
    let mut stack = Stack::new().expect("Stack::new");
    let ctx = req_bld.x509v3_context(None);
    let mut an = SubjectAlternativeName::new();
    an.dns("*.k37713.xyz");

    let ext = an.build(&ctx).expect("SubjectAlternativeName::build");
    stack.push(ext).expect("Stack::push");
    req_bld.add_extensions(&stack).expect("add_extensions");

    // sign it
    req_bld
        .sign(pkey, MessageDigest::sha256())
        .expect("csr_sign");

    // the csr
    Ok(req_bld.build())
}