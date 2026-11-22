//! Who the person is to Phiora, and what they can do about it, as the
//! web's account does (`puikit/app/phi-kit/src/auth.ts`, `account.ts`):
//! be one of the members Phiora lets anyone be to try things; sign in
//! with a handle and a password; be nobody again; see the sessions
//! that are signed in as them and sign the others out; and take out
//! everything Phiora keeps about them. Each is asked at once, on the
//! person's own session, and the answer is what is shown. What each is
//! asked by, and the names in what is sent and answered, are
//! `data/keep.tsv`.

use serde_json::{Value, json};

use super::Kept;

impl Kept {
    /// Who the person is, as Phiora says it; nothing where it does not
    /// answer.
    pub fn me(&self) -> Value {
        self.me.borrow().clone()
    }

    /// A word of `data/keep.tsv`: a name in what Phiora says.
    pub fn word(&self, name: &str) -> String {
        self.word.get(name).to_string()
    }

    /// Whether the person is nobody Phiora knows by name.
    pub fn guest(&self) -> bool {
        self.me()[self.word.get("me.guest")].as_bool().unwrap_or(true)
    }

    /// The members Phiora lets anyone be, to try things: the id, the
    /// name and the handle of each. None where it lets nobody.
    pub fn demo(&self) -> Vec<(String, String, String)> {
        let asked = self.phiora.borrow().as_ref().and_then(|phiora| phiora.get(self.word.get("ask.list")));
        let list = asked.and_then(|text| serde_json::from_str::<Vec<Value>>(&text).ok()).unwrap_or_default();
        let said = |one: &Value, name: &str| one[self.word.get(name)].as_str().unwrap_or_default().to_string();
        list.iter().map(|one| (said(one, "me.id"), said(one, "me.name"), said(one, "me.handle"))).collect()
    }

    /// Be one of those members; or, with nobody named, be nobody
    /// again: a guest, with a session of their own. `false` when
    /// Phiora did not do it, and nothing changes.
    pub fn sign(&self, user: Option<&str>) -> bool {
        let Some(mut phiora) = self.phiora.borrow().clone() else { return false };
        // What changed so far goes first, under who the person was.
        self.settle();
        let done = match user {
            Some(id) => phiora.post(self.word.get("ask.demo"), &json!({ self.word.get("demo.user"): id }).to_string()),
            None => {
                let left = phiora.post(self.word.get("ask.out"), "{}");
                phiora.cookie.clear();
                left && phiora.enter()
            }
        };
        if done {
            *self.phiora.borrow_mut() = Some(phiora);
        }
        self.fetch(done);
        done
    }

    /// Sign in with a handle and a password. What Phiora said when it
    /// refused, in its own words; the password is sent and not kept.
    pub fn enter(&self, handle: &str, secret: &str) -> Result<(), String> {
        let Some(phiora) = self.phiora.borrow().clone() else { return Err(String::new()) };
        self.settle();
        let body = json!({ self.word.get("signin.handle"): handle.trim(), self.word.get("signin.secret"): secret });
        let (done, said) = phiora.said(self.word.get("ask.signin"), &body.to_string()).unwrap_or_default();
        self.fetch(done);
        match done {
            true => Ok(()),
            false => Err(serde_json::from_str::<Value>(&said).ok().and_then(|said| said[self.word.get("said.message")].as_str().map(str::to_string)).unwrap_or_default()),
        }
    }

    /// The sessions signed in as the person: what each is on, and
    /// whether it is this one.
    pub fn session(&self) -> Vec<(String, bool)> {
        let asked = self.phiora.borrow().as_ref().and_then(|phiora| phiora.get(self.word.get("ask.session")));
        let list = asked.and_then(|text| serde_json::from_str::<Vec<Value>>(&text).ok()).unwrap_or_default();
        list.iter().map(|one| (one[self.word.get("session.on")].as_str().unwrap_or_default().to_string(), one[self.word.get("session.this")].as_bool().unwrap_or(false))).collect()
    }

    /// Sign every other session out. `false` when Phiora did not.
    pub fn alone(&self) -> bool {
        let body = json!({ self.word.get("revoke.other"): true }).to_string();
        self.phiora.borrow().as_ref().is_some_and(|phiora| phiora.post(self.word.get("ask.revoke"), &body))
    }

    /// Everything Phiora keeps about the person, as it writes it.
    pub fn export(&self) -> Option<String> {
        self.phiora.borrow().as_ref().and_then(|phiora| phiora.get(self.word.get("ask.export")))
    }
}
