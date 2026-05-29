use argon2::password_hash::{SaltString, rand_core::OsRng};
use argon2::{Algorithm, Argon2, Params, PasswordHash, PasswordHasher, PasswordVerifier, Version};

/// Argon2id memory cost (KiB) — tuned for serverless cold starts.
const ARGON2_M_COST_KIB: u32 = 19_456;
const ARGON2_T_COST: u32 = 2;
const ARGON2_P_COST: u32 = 1;

fn argon2() -> Result<Argon2<'static>, argon2::Error> {
    let params = Params::new(ARGON2_M_COST_KIB, ARGON2_T_COST, ARGON2_P_COST, None)?;
    Ok(Argon2::new(Algorithm::Argon2id, Version::V0x13, params))
}

pub fn hash_password(password: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let hash = argon2()?.hash_password(password.as_bytes(), &salt)?;
    Ok(hash.to_string())
}

pub fn verify_password(password: &str, password_hash: &str) -> bool {
    let Ok(parsed) = PasswordHash::new(password_hash) else {
        return false;
    };
    argon2()
        .ok()
        .and_then(|a| a.verify_password(password.as_bytes(), &parsed).ok())
        .is_some()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("test-password-123").expect("hash");
        assert!(verify_password("test-password-123", &hash));
        assert!(!verify_password("wrong", &hash));
    }
}
