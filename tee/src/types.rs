// Copyright 2022 Capsule Corp (France) SAS.
// This file is part of Ternoa.

// Ternoa is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Ternoa is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Ternoa.  If not, see <http://www.gnu.org/licenses/>.

use parity_scale_codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;
use sp_std::vec::Vec;

pub type EnclaveId = u32;
pub type ClusterId = u32;
pub type ProviderId = u32;
pub type EnclaveOperatorId = u32;

// ********************************************************************
pub const IAS_QUOTE_STATUS_LEVEL_1: &[&str] = &["OK"];
pub const IAS_QUOTE_STATUS_LEVEL_2: &[&str] = &["SW_HARDENING_NEEDED"];
pub const IAS_QUOTE_STATUS_LEVEL_3: &[&str] =
	&["CONFIGURATION_NEEDED", "CONFIGURATION_AND_SW_HARDENING_NEEDED"];
pub const IAS_QUOTE_STATUS_LEVEL_5: &[&str] = &["GROUP_OUT_OF_DATE"];
pub const IAS_QUOTE_ADVISORY_ID_WHITELIST: &[&str] =
	&["INTEL-SA-00334", "INTEL-SA-00219", "INTEL-SA-00381", "INTEL-SA-00389"];
pub type SignatureAlgorithms = &'static [&'static webpki::SignatureAlgorithm];
pub static SUPPORTED_SIG_ALGS: SignatureAlgorithms = &[
	// &webpki::ECDSA_P256_SHA256,
	// &webpki::ECDSA_P256_SHA384,
	// &webpki::ECDSA_P384_SHA256,
	// &webpki::ECDSA_P384_SHA384,
	&webpki::RSA_PKCS1_2048_8192_SHA256,
	&webpki::RSA_PKCS1_2048_8192_SHA384,
	&webpki::RSA_PKCS1_2048_8192_SHA512,
	&webpki::RSA_PKCS1_3072_8192_SHA384,
];
pub static IAS_SERVER_ROOTS: webpki::TlsServerTrustAnchors = webpki::TlsServerTrustAnchors(&[
	/*
     * -----BEGIN CERTIFICATE-----
     * MIIFSzCCA7OgAwIBAgIJANEHdl0yo7CUMA0GCSqGSIb3DQEBCwUAMH4xCzAJBgNV
     * BAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwLU2FudGEgQ2xhcmExGjAYBgNV
     * BAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQDDCdJbnRlbCBTR1ggQXR0ZXN0
     * YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwIBcNMTYxMTE0MTUzNzMxWhgPMjA0OTEy
     * MzEyMzU5NTlaMH4xCzAJBgNVBAYTAlVTMQswCQYDVQQIDAJDQTEUMBIGA1UEBwwL
     * U2FudGEgQ2xhcmExGjAYBgNVBAoMEUludGVsIENvcnBvcmF0aW9uMTAwLgYDVQQD
     * DCdJbnRlbCBTR1ggQXR0ZXN0YXRpb24gUmVwb3J0IFNpZ25pbmcgQ0EwggGiMA0G
     * CSqGSIb3DQEBAQUAA4IBjwAwggGKAoIBgQCfPGR+tXc8u1EtJzLA10Feu1Wg+p7e
     * LmSRmeaCHbkQ1TF3Nwl3RmpqXkeGzNLd69QUnWovYyVSndEMyYc3sHecGgfinEeh
     * rgBJSEdsSJ9FpaFdesjsxqzGRa20PYdnnfWcCTvFoulpbFR4VBuXnnVLVzkUvlXT
     * L/TAnd8nIZk0zZkFJ7P5LtePvykkar7LcSQO85wtcQe0R1Raf/sQ6wYKaKmFgCGe
     * NpEJUmg4ktal4qgIAxk+QHUxQE42sxViN5mqglB0QJdUot/o9a/V/mMeH8KvOAiQ
     * byinkNndn+Bgk5sSV5DFgF0DffVqmVMblt5p3jPtImzBIH0QQrXJq39AT8cRwP5H
     * afuVeLHcDsRp6hol4P+ZFIhu8mmbI1u0hH3W/0C2BuYXB5PC+5izFFh/nP0lc2Lf
     * 6rELO9LZdnOhpL1ExFOq9H/B8tPQ84T3Sgb4nAifDabNt/zu6MmCGo5U8lwEFtGM
     * RoOaX4AS+909x00lYnmtwsDVWv9vBiJCXRsCAwEAAaOByTCBxjBgBgNVHR8EWTBX
     * MFWgU6BRhk9odHRwOi8vdHJ1c3RlZHNlcnZpY2VzLmludGVsLmNvbS9jb250ZW50
     * L0NSTC9TR1gvQXR0ZXN0YXRpb25SZXBvcnRTaWduaW5nQ0EuY3JsMB0GA1UdDgQW
     * BBR4Q3t2pn680K9+QjfrNXw7hwFRPDAfBgNVHSMEGDAWgBR4Q3t2pn680K9+Qjfr
     * NXw7hwFRPDAOBgNVHQ8BAf8EBAMCAQYwEgYDVR0TAQH/BAgwBgEB/wIBADANBgkq
     * hkiG9w0BAQsFAAOCAYEAeF8tYMXICvQqeXYQITkV2oLJsp6J4JAqJabHWxYJHGir
     * IEqucRiJSSx+HjIJEUVaj8E0QjEud6Y5lNmXlcjqRXaCPOqK0eGRz6hi+ripMtPZ
     * sFNaBwLQVV905SDjAzDzNIDnrcnXyB4gcDFCvwDFKKgLRjOB/WAqgscDUoGq5ZVi
     * zLUzTqiQPmULAQaB9c6Oti6snEFJiCQ67JLyW/E83/frzCmO5Ru6WjU4tmsmy8Ra
     * Ud4APK0wZTGtfPXU7w+IBdG5Ez0kE1qzxGQaL4gINJ1zMyleDnbuS8UicjJijvqA
     * 152Sq049ESDz+1rRGc2NVEqh1KaGXmtXvqxXcTB+Ljy5Bw2ke0v8iGngFBPqCTVB
     * 3op5KBG3RjbF6RRSzwzuWfL7QErNC8WEy5yDVARzTA5+xmBc388v9Dm21HGfcC8O
     * DD+gT9sSpssq0ascmvH49MOgjt1yoysLtdCtJW/9FZpoOypaHx0R+mJTLwPXVMrv
     * DaVzWh5aiEx+idkSGMnX
     * -----END CERTIFICATE-----
     */
	webpki::TrustAnchor {
		subject: b"1\x0b0\t\x06\x03U\x04\x06\x13\x02US1\x0b0\t\x06\x03U\x04\x08\x0c\x02CA1\x140\x12\x06\x03U\x04\x07\x0c\x0bSanta Clara1\x1a0\x18\x06\x03U\x04\n\x0c\x11Intel Corporation100.\x06\x03U\x04\x03\x0c\'Intel SGX Attestation Report Signing CA",
		spki: b"0\r\x06\t*\x86H\x86\xf7\r\x01\x01\x01\x05\x00\x03\x82\x01\x8f\x000\x82\x01\x8a\x02\x82\x01\x81\x00\x9f<d~\xb5w<\xbbQ-\'2\xc0\xd7A^\xbbU\xa0\xfa\x9e\xde.d\x91\x99\xe6\x82\x1d\xb9\x10\xd51w7\twFjj^G\x86\xcc\xd2\xdd\xeb\xd4\x14\x9dj/c%R\x9d\xd1\x0c\xc9\x877\xb0w\x9c\x1a\x07\xe2\x9cG\xa1\xae\x00IHGlH\x9fE\xa5\xa1]z\xc8\xec\xc6\xac\xc6E\xad\xb4=\x87g\x9d\xf5\x9c\t;\xc5\xa2\xe9ilTxT\x1b\x97\x9euKW9\x14\xbeU\xd3/\xf4\xc0\x9d\xdf\'!\x994\xcd\x99\x05\'\xb3\xf9.\xd7\x8f\xbf)$j\xbe\xcbq$\x0e\xf3\x9c-q\x07\xb4GTZ\x7f\xfb\x10\xeb\x06\nh\xa9\x85\x80!\x9e6\x91\tRh8\x92\xd6\xa5\xe2\xa8\x08\x03\x19>@u1@N6\xb3\x15b7\x99\xaa\x82Pt@\x97T\xa2\xdf\xe8\xf5\xaf\xd5\xfec\x1e\x1f\xc2\xaf8\x08\x90o(\xa7\x90\xd9\xdd\x9f\xe0`\x93\x9b\x12W\x90\xc5\x80]\x03}\xf5j\x99S\x1b\x96\xdei\xde3\xed\"l\xc1 }\x10B\xb5\xc9\xab\x7f@O\xc7\x11\xc0\xfeGi\xfb\x95x\xb1\xdc\x0e\xc4i\xea\x1a%\xe0\xff\x99\x14\x88n\xf2i\x9b#[\xb4\x84}\xd6\xff@\xb6\x06\xe6\x17\x07\x93\xc2\xfb\x98\xb3\x14X\x7f\x9c\xfd%sb\xdf\xea\xb1\x0b;\xd2\xd9vs\xa1\xa4\xbdD\xc4S\xaa\xf4\x7f\xc1\xf2\xd3\xd0\xf3\x84\xf7J\x06\xf8\x9c\x08\x9f\r\xa6\xcd\xb7\xfc\xee\xe8\xc9\x82\x1a\x8eT\xf2\\\x04\x16\xd1\x8cF\x83\x9a_\x80\x12\xfb\xdd=\xc7M%by\xad\xc2\xc0\xd5Z\xffo\x06\"B]\x1b\x02\x03\x01\x00\x01",
		name_constraints: None
	},
]);
#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum AttestationReport {
	SgxIas { ra_report: Vec<u8>, signature: Vec<u8>, raw_signing_cert: Vec<u8> },
}
#[cfg_attr(feature = "enable_serde", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo, Debug, Copy, Clone, PartialEq, Eq)]
pub enum AttestationProvider {
	#[cfg_attr(feature = "enable_serde", serde(rename = "root"))]
	Root,
	#[cfg_attr(feature = "enable_serde", serde(rename = "ias"))]
	Ias,
}

#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub struct ConfidentialReport {
	pub confidence_level: u8,
	pub provider: Option<AttestationProvider>,
	pub runtime_hash: Vec<u8>,
}
#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub enum ReportError {
	PRuntimeRejected,
	InvalidIASSigningCert,
	InvalidReport,
	InvalidQuoteStatus,
	BadIASReport,
	OutdatedIASReport,
	UnknownQuoteBodyFormat,
	InvalidUserDataHash,
	NoneAttestationDisabled,
}
#[derive(Encode, Decode, TypeInfo, Debug, Clone, PartialEq, Eq)]
pub struct IasFields {
	pub mr_enclave: [u8; 32],
	pub mr_signer: [u8; 32],
	pub isv_prod_id: [u8; 2],
	pub isv_svn: [u8; 2],
	pub report_data: [u8; 64],
	pub confidence_level: u8,
}

impl IasFields {
	pub fn from_ias_report(report: &[u8]) -> Result<(IasFields, i64), ReportError> {
		// Validate related fields
		let parsed_report: serde_json::Value =
			serde_json::from_slice(report).or(Err(ReportError::InvalidReport))?;

		// Extract quote fields
		let raw_quote_body = parsed_report["isvEnclaveQuoteBody"]
			.as_str()
			.ok_or(ReportError::UnknownQuoteBodyFormat)?;
		let quote_body =
			base64::decode(raw_quote_body).or(Err(ReportError::UnknownQuoteBodyFormat))?;
		let mr_enclave = &quote_body[112..144];
		let mr_signer = &quote_body[176..208];
		let isv_prod_id = &quote_body[304..306];
		let isv_svn = &quote_body[306..308];
		let report_data = &quote_body[368..432];

		// Extract report time
		let raw_report_timestamp =
			parsed_report["timestamp"].as_str().unwrap_or("UNKNOWN").to_owned() + "Z";
		let report_timestamp = chrono::DateTime::parse_from_rfc3339(&raw_report_timestamp)
			.or(Err(ReportError::BadIASReport))?
			.timestamp();

		// Filter valid `isvEnclaveQuoteStatus`
		let quote_status = &parsed_report["isvEnclaveQuoteStatus"].as_str().unwrap_or("UNKNOWN");
		let mut confidence_level: u8 = 128;
		if IAS_QUOTE_STATUS_LEVEL_1.contains(quote_status) {
			confidence_level = 1;
		} else if IAS_QUOTE_STATUS_LEVEL_2.contains(quote_status) {
			confidence_level = 2;
		} else if IAS_QUOTE_STATUS_LEVEL_3.contains(quote_status) {
			confidence_level = 3;
		} else if IAS_QUOTE_STATUS_LEVEL_5.contains(quote_status) {
			confidence_level = 5;
		}
		if confidence_level == 128 {
			return Err(ReportError::InvalidQuoteStatus)
		}
		if confidence_level < 5 {
			// Filter AdvisoryIDs. `advisoryIDs` is optional
			if let Some(advisory_ids) = parsed_report["advisoryIDs"].as_array() {
				for advisory_id in advisory_ids {
					let advisory_id = advisory_id.as_str().ok_or(ReportError::BadIASReport)?;
					if !IAS_QUOTE_ADVISORY_ID_WHITELIST.contains(&advisory_id) {
						confidence_level = 4;
					}
				}
			}
		}

		Ok((
			IasFields {
				mr_enclave: mr_enclave.try_into().unwrap(),
				mr_signer: mr_signer.try_into().unwrap(),
				isv_prod_id: isv_prod_id.try_into().unwrap(),
				isv_svn: isv_svn.try_into().unwrap(),
				report_data: report_data.try_into().unwrap(),
				confidence_level,
			},
			report_timestamp,
		))
	}

	pub fn extend_mrenclave(&self) -> Vec<u8> {
		let mut t_mrenclave = Vec::new();
		t_mrenclave.extend_from_slice(&self.mr_enclave);
		t_mrenclave.extend_from_slice(&self.isv_prod_id);
		t_mrenclave.extend_from_slice(&self.isv_svn);
		t_mrenclave.extend_from_slice(&self.mr_signer);
		t_mrenclave
	}
}

// ********************************************************************

#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
pub struct EnclaveProvider {
	pub enclave_provider_name: Vec<u8>,
}

impl EnclaveProvider {
	pub fn new(enclave_provider_name: Vec<u8>) -> Self {
		Self { enclave_provider_name }
	}
}

// account_id | enclave_id | enclave_class | public_key
#[derive(Encode, Decode, Clone, Eq, PartialEq, RuntimeDebug, TypeInfo, Default)]
pub struct EnclaveProviderKeys<AccountId> {
	pub enclave_class: Option<Vec<u8>>,
	pub account_id: AccountId,
	pub public_key: Vec<u8>,
}

impl<AccountId> EnclaveProviderKeys<AccountId> {
	pub fn new(enclave_class: Option<Vec<u8>>, account_id: AccountId, public_key: Vec<u8>) -> Self {
		Self { enclave_class, account_id, public_key }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Enclave {
	pub api_uri: Vec<u8>,
}

impl Enclave {
	pub fn new(api_uri: Vec<u8>) -> Self {
		Self { api_uri }
	}
}

#[derive(Encode, Decode, Clone, PartialEq, Eq, RuntimeDebug, TypeInfo)]
pub struct Cluster {
	pub enclaves: Vec<EnclaveId>,
}

impl Cluster {
	pub fn new(enclaves: Vec<EnclaveId>) -> Self {
		Self { enclaves }
	}
}
