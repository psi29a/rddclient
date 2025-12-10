use std::error::Error;
use std::net::IpAddr;

pub mod afraid;
pub mod changeip;
pub mod cloudflare;
pub mod cloudns;
pub mod ddnsfm;
pub mod ddnss;
pub mod desec;
pub mod digitalocean;
pub mod dinahosting;
pub mod directnic;
pub mod dnsexit2;
pub mod dnsmadeeasy;
pub mod dnspod;
pub mod domeneshop;
pub mod dondominio;
pub mod dslreports1;
pub mod duckdns;
pub mod dyndns1;
pub mod dyndns2;
pub mod dynu;
pub mod easydns;
pub mod emailonly;
pub mod enom;
pub mod freedns;
pub mod freemyip;
pub mod gandi;
pub mod godaddy;
pub mod googledomains;
pub mod he;
pub mod hetzner;
pub mod infomaniak;
pub mod inwx;
pub mod keysystems;
pub mod linode;
pub mod loopia;
pub mod luadns;
pub mod mythicbeasts;
pub mod namecheap;
pub mod nfsn;
pub mod njalla;
pub mod noip;
pub mod nsupdate;
pub mod one984;
pub mod ovh;
pub mod porkbun;
pub mod regfish;
pub mod selfhost;
pub mod sitelutions;
pub mod woima;
pub mod yandex;
pub mod zoneedit;
pub mod zoneedit1;

/// Common trait that all DNS client implementations must implement
pub trait DnsClient {
    /// Update DNS record with the provided IP address
    /// Returns Ok(()) on success, or an error if the update fails
    fn update_record(&self, hostname: &str, ip: IpAddr) -> Result<(), Box<dyn Error>>;

    /// Validate that the client has all required configuration
    fn validate_config(&self) -> Result<(), Box<dyn Error>>;

    /// Get the provider name for logging purposes
    fn provider_name(&self) -> &str;
}

/// Factory function to create the appropriate DNS client based on provider type
pub fn create_client(provider: &str, config: &crate::config::Config) -> Result<Box<dyn DnsClient>, Box<dyn Error>> {
    // Normalize provider name to lowercase ASCII once for consistent matching
    let normalized = provider.to_ascii_lowercase();
    match normalized.as_str() {
        "1984" | "one984" => Ok(Box::new(one984::One984Client::new(config)?)),
        "afraid" => Ok(Box::new(afraid::AfraidClient::new(config)?)),
        "changeip" => Ok(Box::new(changeip::ChangeipClient::new(config)?)),
        "cloudflare" => Ok(Box::new(cloudflare::CloudflareClient::new(config)?)),
        "cloudns" => Ok(Box::new(cloudns::CloudnsClient::new(config)?)),
        // "cloudxns" - REMOVED: Service defunct, DNS provider shut down
        "ddnsfm" | "ddns.fm" => Ok(Box::new(ddnsfm::DdnsfmClient::new(config)?)),
        "ddnss" => Ok(Box::new(ddnss::DdnssClient::new(config)?)),
        "desec" => Ok(Box::new(desec::DesecClient::new(config)?)),
        "digitalocean" => Ok(Box::new(digitalocean::DigitalOceanClient::new(config)?)),
        "dinahosting" => Ok(Box::new(dinahosting::DinahostingClient::new(config)?)),
        "directnic" => Ok(Box::new(directnic::DirectnicClient::new(config)?)),
        "dnsexit" | "dnsexit2" => Ok(Box::new(dnsexit2::Dnsexit2Client::new(config)?)),
        "dnsmadeeasy" | "dns-made-easy" => Ok(Box::new(dnsmadeeasy::DnsMadeEasyClient::new(config)?)),
        "dnspod" => Ok(Box::new(dnspod::DnspodClient::new(config)?)),
        "domeneshop" => Ok(Box::new(domeneshop::DomeneshopClient::new(config)?)),
        "dondominio" => Ok(Box::new(dondominio::DonDominioClient::new(config)?)),
        "dslreports" | "dslreports1" => Ok(Box::new(dslreports1::Dslreports1Client::new(config)?)),
        "duckdns" => Ok(Box::new(duckdns::DuckDnsClient::new(config)?)),
        "dyndns1" => Ok(Box::new(dyndns1::Dyndns1Client::new(config)?)),
        "dyndns2" | "dyndns" => Ok(Box::new(dyndns2::DynDns2Client::new(config)?)),
        "dynu" => Ok(Box::new(dynu::DynuClient::new(config)?)),
        "easydns" => Ok(Box::new(easydns::EasydnsClient::new(config)?)),
        "emailonly" => Ok(Box::new(emailonly::EmailonlyClient::new(config)?)),
        "enom" => Ok(Box::new(enom::EnomClient::new(config)?)),
        "freedns" => Ok(Box::new(freedns::FreednsClient::new(config)?)),
        "freemyip" => Ok(Box::new(freemyip::FreemyipClient::new(config)?)),
        "gandi" => Ok(Box::new(gandi::GandiClient::new(config)?)),
        "godaddy" => Ok(Box::new(godaddy::GoDaddyClient::new(config)?)),
        "googledomains" | "google-domains" => Ok(Box::new(googledomains::GoogleDomainsClient::new(config)?)),
        "he" | "hurricane" | "hurricaneelectric" => Ok(Box::new(he::HurricaneElectricClient::new(config)?)),
        "hetzner" => Ok(Box::new(hetzner::HetznerClient::new(config)?)),
        "infomaniak" => Ok(Box::new(infomaniak::InfomaniakClient::new(config)?)),
        "inwx" => Ok(Box::new(inwx::InwxClient::new(config)?)),
        "keysystems" | "key-systems" => Ok(Box::new(keysystems::KeysystemsClient::new(config)?)),
        "linode" => Ok(Box::new(linode::LinodeClient::new(config)?)),
        "loopia" => Ok(Box::new(loopia::LoopiaClient::new(config)?)),
        "luadns" => Ok(Box::new(luadns::LuadnsClient::new(config)?)),
        "mythicbeasts" | "mythic-beasts" | "mythicdyn" => Ok(Box::new(mythicbeasts::MythicbeastsClient::new(config)?)),
        "namecheap" => Ok(Box::new(namecheap::NamecheapClient::new(config)?)),
        "nfsn" => Ok(Box::new(nfsn::NfsnClient::new(config)?)),
        "njalla" => Ok(Box::new(njalla::NjallaClient::new(config)?)),
        "noip" | "no-ip" => Ok(Box::new(noip::NoIpClient::new(config)?)),
        "nsupdate" => Ok(Box::new(nsupdate::NsupdateClient::new(config)?)),
        "ovh" => Ok(Box::new(ovh::OvhClient::new(config)?)),
        "porkbun" => Ok(Box::new(porkbun::PorkbunClient::new(config)?)),
        "regfish" => Ok(Box::new(regfish::RegfishClient::new(config)?)),
        "selfhost" => Ok(Box::new(selfhost::SelfhostClient::new(config)?)),
        "sitelutions" => Ok(Box::new(sitelutions::SitelutionsClient::new(config)?)),
        "woima" => Ok(Box::new(woima::WoimaClient::new(config)?)),
        "yandex" => Ok(Box::new(yandex::YandexClient::new(config)?)),
        "zoneedit" => Ok(Box::new(zoneedit::ZoneeditClient::new(config)?)),
        "zoneedit1" => Ok(Box::new(zoneedit1::Zoneedit1Client::new(config)?)),
        _ => Err(format!("Unsupported provider: {}. Supported providers: 1984/one984, afraid, changeip, cloudflare, cloudns, ddnsfm/ddns.fm, ddnss, desec, digitalocean, dinahosting, directnic, dnsexit/dnsexit2, dnsmadeeasy/dns-made-easy, dnspod, domeneshop, dondominio, dslreports/dslreports1, duckdns, dyndns1, dyndns/dyndns2, dynu, easydns, emailonly, enom, freedns, freemyip, gandi, godaddy, googledomains/google-domains, he/hurricane/hurricaneelectric, hetzner, infomaniak, inwx, keysystems/key-systems, linode, loopia, luadns, mythicbeasts/mythic-beasts/mythicdyn, namecheap, nfsn, njalla, noip/no-ip, nsupdate, ovh, porkbun, regfish, selfhost, sitelutions, woima, yandex, zoneedit, zoneedit1", provider).into()),
    }
}
