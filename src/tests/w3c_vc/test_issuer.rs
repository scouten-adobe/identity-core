// Copyright 2024 Adobe. All rights reserved.
// This file is licensed to you under the Apache License,
// Version 2.0 (http://www.apache.org/licenses/LICENSE-2.0)
// or the MIT license (http://opensource.org/licenses/MIT),
// at your option.

// Unless required by applicable law or agreed to in writing,
// this software is distributed on an "AS IS" BASIS, WITHOUT
// WARRANTIES OR REPRESENTATIONS OF ANY KIND, either express or
// implied. See the LICENSE-MIT and LICENSE-APACHE files for the
// specific language governing permissions and limitations under
// each license.

#![allow(unused)] // TEMPORARY while rebuilding
#![allow(dead_code)] // TEMPORARY while rebuilding

use std::{fs::OpenOptions, io::Cursor, str::FromStr};

use c2pa::{Manifest, ManifestStore};
use iref::UriBuf;
use ssi::{
    claims::{
        vc::{
            syntax::{IdOr, NonEmptyVec},
            v2::Credential,
        },
        vc_jose_cose::JoseVc,
    },
    dids::DIDJWK,
    JWK,
};
use static_iref::uri;
use xsd_types::value::DateTimeStamp;

use crate::{
    builder::{CredentialHolder, IdentityAssertionBuilder, ManifestBuilder},
    tests::fixtures::{temp_c2pa_signer, temp_dir_path},
    w3c_vc::{CreatorIdentityAssertion, IdentityAssertionVc, IdentityProvider, VerifiedIdentity},
    IdentityAssertion, SignerPayload,
};

/// TO DO: Move what we can from this to more generic code in pub mod w3c_vc.
pub(super) struct TestIssuer {
    setup: TestSetup,
}

enum TestSetup {
    UserAndIssuerJwk(JWK, JWK),
    // Credential(Credential), // redo for ssi 0.8.0
}

#[async_trait::async_trait]
impl CredentialHolder for TestIssuer {
    fn sig_type(&self) -> &'static str {
        "cawg.w3c.vc"
    }

    fn reserve_size(&self) -> usize {
        10240 // 🤷🏻‍♂️
    }

    async fn sign(&self, _signer_payload: &SignerPayload) -> c2pa::Result<Vec<u8>> {
        // TO DO: ERROR HANDLING
        let signed_vc = match &self.setup {
            TestSetup::UserAndIssuerJwk(user_jwk, issuer_jwk) => {
                // WARNING: did:jwk is great for simple test cases such as this
                // but is strongly discouraged for production use cases. In other words,
                // please don't copy and paste this into your own implementation!

                let user_did = DIDJWK::generate_url(&user_jwk.to_public());
                let issuer_did = DIDJWK::generate_url(&issuer_jwk.to_public());

                // Use the identities as shown in https://creator-assertions.github.io/identity/1.x+vc-draft/#vc-credentialsubject-verifiedIdentities.

                let verified_identities = vec![
                    VerifiedIdentity {
                        type_: "cawg.document_verification".to_owned(),
                        name: Some("First-Name Last-Name".to_owned()),
                        username: None,
                        address: None,
                        uri: None,
                        provider: IdentityProvider {
                            id: UriBuf::from_str("https://example-id-verifier.com").unwrap(),
                            name: "Example ID Verifier".to_owned(),
                            // "proof": "https://example-id-verifier.com/proofs/1"
                        },
                        verified_at: DateTimeStamp::from_str("2024-07-26T22:30:15Z").unwrap(),
                    },
                    VerifiedIdentity {
                        type_: "cawg.affiliation".to_owned(),
                        name: None,
                        username: None,
                        address: None,
                        uri: None,
                        provider: IdentityProvider {
                            id: UriBuf::from_str("https://example-affiliated-organization.com")
                                .unwrap(),
                            name: "Example Affiliated Organization".to_owned(),
                            // "proof": "https://example-affiliated-organization.com/proofs/ck4592p5lk8u05mdg8bg5ac7ishlqfh1"
                        },
                        verified_at: DateTimeStamp::from_str("2024-07-26T22:29:57Z").unwrap(),
                    },
                    VerifiedIdentity {
                        type_: "cawg.social_media".to_owned(),
                        name: Some("Silly Cats 929".to_owned()),
                        username: Some("username".to_owned()),
                        address: None,
                        uri: Some(UriBuf::from_str("https://example-social-network.com/username").unwrap()),
                        provider: IdentityProvider {
                            id: UriBuf::from_str("https://example-social-network.com")
                                .unwrap(),
                            name: "Example Social Network".to_owned(),
                        },
                        verified_at: DateTimeStamp::from_str("2024-05-27T08:40:39.569856Z").unwrap(),
                    },
                    VerifiedIdentity {
                        type_: "cawg.crypto_wallet".to_owned(),
                        name: None,
                        username: None,
                        address: Some("fa64ef445f994138bdeb9baac6ce1e16".to_owned()),
                        uri: Some(UriBuf::from_str("https://example-crypto-wallet.com/fa64ef445f994138bdeb9baac6ce1e16").unwrap()),
                        provider: IdentityProvider {
                            id: UriBuf::from_str("https://example-crypto-wallet.com")
                                .unwrap(),
                            name: "Example Crypto Wallet".to_owned(),
                        },
                        verified_at: DateTimeStamp::from_str("2024-05-27T08:40:39.569856Z").unwrap(),
                    },
                ];

                let cia = CreatorIdentityAssertion {
                    verified_identities,
                };

                let subjects = NonEmptyVec::new(cia);

                let mut asset_vc = IdentityAssertionVc::new(
                    None,
                    IdOr::Id(issuer_did.clone().into_uri()),
                    subjects,
                );

                asset_vc.valid_from = Some(DateTimeStamp::now());

                dbg!(&asset_vc);

                // TO DO: Switch to COSE once available.
                let jose_vc = JoseVc(asset_vc);
                let jose = jose_vc.sign_into_enveloped(&issuer_jwk).await.unwrap();

                dbg!(&jose);
                panic!("Now what?");

                // See example at https://docs.rs/ssi/latest/ssi/index.html.

                /* TO DO: Rework proof once I grok the new ssi APIs.

                let mut context_loader = cawg_context_loader();
                asset_vc.add_proof(
                    asset_vc
                        .generate_proof(
                            &issuer_jwk,
                            &LinkedDataProofOptions::default(),
                            &DIDKey,
                            &mut context_loader,
                        )
                        .await
                        .unwrap(),
                );
                */

                // asset_vc
            } // TestSetup::Credential(vc) => vc.clone(),
        };

        /*
        eprintln!(
            "\n\n\nAsset VC is\n{}\n\n",
            serde_json::to_string_pretty(&signed_vc).unwrap()
        );

        unimplemented!("Rebuild for ssi 0.8.0");
        /x*
        let asset_vc = serde_json::to_string(&asset_vc)?;
        Ok(asset_vc.as_bytes().to_owned())
        */
    }
}

impl TestIssuer {
    pub(super) fn new() -> Self {
        Self {
            setup: TestSetup::UserAndIssuerJwk(
                JWK::generate_ed25519().unwrap(),
                JWK::generate_ed25519().unwrap(),
            ),
        }
    }

    pub(super) fn from_asset_vc(_asset_vc_json: &str) -> Self {
        unimplemented!("Rebuild for ssi 0.8.0");
        /*
        let vc = Credential::from_json(asset_vc_json).unwrap();
        Self {
            setup: TestSetup::Credential(vc),
        }
        */
    }

    pub(super) async fn test_basic_case(self) {
        // TO DO: See if we can make this a non-consuming function.
        // Currently does so because IdentityAssertionBuilder takes
        // ownership of the CredentialHolder instance.

        // TO DO: Clean up code and extract into builder interface.
        // For now, just looking for a simple proof-of-concept.

        let cloud_jpg = include_bytes!("../fixtures/cloud.jpg");
        let mut input_stream = Cursor::new(cloud_jpg);

        let temp_dir = tempfile::tempdir().unwrap();
        let dest = temp_dir_path(&temp_dir, "cloud_output.jpg");

        let mut output_stream = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&dest)
            .unwrap();

        // TO DO: Add a metadata assertion as an example.

        // Here we act as an identity assertion creator.

        let iab = IdentityAssertionBuilder::for_credential_holder(self);

        let signer = temp_c2pa_signer();
        let mut mb = ManifestBuilder::default();
        mb.add_assertion(iab);

        let manifest: Manifest = Manifest::new("identity_test/simple_case");
        mb.build(
            manifest,
            "jpg",
            &mut input_stream,
            &mut output_stream,
            signer.as_ref(),
        )
        .await
        .unwrap();

        // Here we act as an identity assertion consumer.

        let manifest_store = ManifestStore::from_file(&dest).unwrap();
        assert!(manifest_store.validation_status().is_none());

        let manifest = manifest_store.get_active().unwrap();
        let identity: IdentityAssertion = manifest.find_assertion("cawg.identity").unwrap();

        let _sp = identity.check_signer_payload(manifest).unwrap();
        identity.check_padding().unwrap();

        let report = identity.validate(manifest).await.unwrap();

        let sp = report.signer_payload;
        let ra = &sp.referenced_assertions;
        assert_eq!(ra.len(), 1);

        let ra1 = ra.first().unwrap();
        assert_eq!(ra1.url, "self#jumbf=c2pa.assertions/c2pa.hash.data");
        assert_eq!(ra1.alg, Some("sha256".to_owned()));

        assert_eq!(report.signer_payload.sig_type, "cawg.w3c.vc");

        dbg!(&report.named_actor);

        unimplemented!();
    }
}
