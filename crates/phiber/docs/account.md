# Phiber — Account & Profile System Reference

> **Location**: `crates/phiber/docs/account.md`
> Based on actual Zed source code research (client/user.rs, title_bar/title_bar.rs, notifications/).

---

## What Zed Actually Has (Real Source Research)

### Zed User Model (client/src/user.rs)

```rust
pub struct User {
    pub legacy_id: LegacyUserId,    // u64
    pub username: SharedString,
    pub avatar_uri: SharedUri,       // URL to avatar image
    pub name: Option<String>,        // display name (optional)
}

pub struct Contact {
    pub user: Arc<User>,
    pub online: bool,
    pub busy: bool,           // in an active call
}

pub enum ContactRequestStatus {
    None,
    RequestSent,
    RequestReceived,
    RequestAccepted,
}
```

### Zed Connection Status (client/src/client.rs)

```rust
pub enum Status {
    SignedOut,
    UpgradeRequired,
    Authenticating,
    Authenticated,
    AuthenticationError,
    Connecting,
    ConnectionError,
    Connected { peer_id: PeerId, connection_id: ConnectionId },
    ConnectionLost,
    Reauthenticating,
    Reauthenticated,
    Reconnecting,
    ReconnectionError { next_reconnection: Instant },
}
```

### Zed UserStore (client/src/user.rs)

```rust
pub struct UserStore {
    users: HashMap<u64, Arc<User>>,
    current_user: watch::Receiver<Option<Arc<User>>>,
    current_organization: Option<Arc<Organization>>,
    organizations: Vec<Arc<Organization>>,
    plans_by_organization: HashMap<OrganizationId, Plan>,
    contacts: Vec<Arc<Contact>>,
    incoming_contact_requests: Vec<Arc<User>>,
    outgoing_contact_requests: Vec<Arc<User>>,
    edit_prediction_usage: Option<EditPredictionUsage>,
    plan_info: Option<PlanInfo>,
}

// Events emitted:
pub enum Event {
    Contact { user: Arc<User>, kind: ContactEventKind },
    ShowContacts,
    ParticipantIndicesChanged,
    PrivateUserInfoUpdated,
    PlanUpdated,
    OrganizationChanged,
}

pub enum ContactEventKind {
    Requested,
    Accepted,
    Cancelled,
}
```

### Zed User Menu (title_bar/src/title_bar.rs — actual render_user_menu_button)

The REAL Zed user menu is a `PopoverMenu` (dropdown), NOT a modal. It contains:

```
[Avatar / Chevron icon]   <- trigger button in titlebar
     |
     v PopoverMenu drops down:
+--------------------------------+
| @username                      |  -> opens zed_urls::account_url
| ─────                          |
| Restart to update Zed [▼]      |  (if update available)
| ─────                          |
| Organization                   |  (header)
|   My Org             [Pro]  ✓  |  (current, with plan chip)
|   Other Org          [Free]    |
| ─────                          |
| Settings                       |  -> OpenSettings action
| Keymap                         |  -> OpenKeymap action
| Themes…                        |  -> theme_selector::Toggle
| Icon Themes…                   |  -> icon_theme_selector::Toggle
| Extensions                     |  -> Extensions action
| ─────                          |
| Panel Layout  >                |  -> submenu (if AI enabled)
|   Classic ○                   |
|   Agentic  ●                   |
|   Custom   ●  (disabled)       |
| ─────                          |
| Sign Out                       |  -> client::SignOut action
+--------------------------------+
```

### Zed Notification System (notifications/src/notification_store.rs)

```rust
pub struct NotificationStore {
    notifications: SumTree<NotificationEntry>,
    loaded_all_notifications: bool,
    ...
}

pub struct NotificationEntry {
    pub id: u64,
    pub notification: Notification,  // rpc::Notification enum
    pub timestamp: OffsetDateTime,
    pub is_read: bool,
    pub response: Option<bool>,
}

pub struct NotificationSummary {
    max_id: u64,
    count: usize,
    unread_count: usize,
}

pub enum NotificationEvent {
    NotificationsUpdated { old_range: Range<usize>, new_count: usize },
    NewNotification { entry: NotificationEntry },
    NotificationRemoved { entry: NotificationEntry },
    NotificationRead { entry: NotificationEntry },
}
```

### Zed Collab Panel (collab_ui/src/collab_panel.rs — 169KB)

The collaboration panel (Discord-like channels/friends list) has:
- Contact requests (incoming / outgoing)
- Online contacts
- Channels (like Discord server channels)
- Active calls / rooms

This is the CLOSEST thing Zed has to Discord's social layer.

---

## Gap Analysis: Zed vs. Phiber Profile Modal

```
FEATURE                  ZED HAS IT    PHIBER CURRENT  PHIBER TARGET
─────────────────────────────────────────────────────────────────────
User.username            YES           stored as str   read from UserStore
User.avatar_uri          YES           initials only   Avatar image + fallback
User.name                YES           no              display name
Online status (bool)     Contact.online no             BStatusType (4 values)
Organization + plan      YES           no              show in banner
Contact list             YES (collab)  no              future (collab panel)
Contact requests         YES           no              future
Notifications            YES           no              future
Activity feed            NO            no              Phiber custom
Custom status message    NO            no              Phiber custom
Sign in                  YES           label only      OAuth flow
Sign out                 YES           label only      client::SignOut action
Settings                 OpenSettings  label only      link to modal
Keymap                   OpenKeymap    label only      link to modal
Theme picker             Toggle action label only      Themes tab in modal
Extensions               Extensions    no              future
Update notification      YES (titlebar) no             future
```

---

## Phiber Profile Modal — Accurate Target Design

Based on the real Zed User model, here is what Phiber's profile modal should actually do:

```
+----------------------------------------------------------+
| PROFILE MODAL (BModalType::Profile, w:440px)             |
|                                                          |
| +--------------------------------------------------------+|
| | SIGNED IN BANNER                                      ||
| |  +----+  {user.name or user.username}  [Account ↗]  ||
| |  |Av  |  @{user.username}                             ||
| |  |atar|  {organization.name}  [{plan chip: Free/Pro}]||
| |  +----+  Connection: Connected / Signing in…          ||
| +--------------------------------------------------------+|
|                                                          |
| CONNECTION STATUS BAR (from client::Status)              |
|  [green dot] Connected     [amber] Reconnecting          |
|  [grey dot] Signed Out     [red] Auth Error              |
|                                                          |
| ======================================================= |
|                                                          |
| ORGANIZATIONS                                            |
|  [check] My Org        [Pro]   <- current               |
|  [ ]     Other Org     [Free]  <- switch org            |
|                                                          |
| ======================================================= |
|                                                          |
| ACTIONS (from actual Zed user menu)                      |
|  [settings] Settings          -> OpenSettings            |
|  [keyboard] Keymap            -> OpenKeymap              |
|  [palette]  Themes...         -> theme_selector::Toggle  |
|  [image]    Icon Themes...    -> icon_theme_selector     |
|  [box]      Extensions        -> Extensions              |
|  [layout]   Panel Layout  >   -> submenu                |
|             Classic / Agentic                            |
|                                                          |
| ======================================================= |
|                                                          |
| NOTIFICATIONS (from NotificationStore)                   |
|  [bell] 3 unread notifications  [View All]               |
|  ─────────────────────────────────────────               |
|  [user] @alice wants to collaborate    2m ago  [Accept]  |
|  [info] Zed updated to v0.182.0       10m ago  [Read]   |
|  [star] New theme available            1h ago  [Read]   |
|                                                          |
| ======================================================= |
|                                                          |
| [logout] Sign Out                                        |
|                                                          |
| +-- SIGNED OUT STATE (when user is None) ---------------+|
| | [login]  Sign In with Zed Account                     ||
| +--------------------------------------------------------+|
+----------------------------------------------------------+
```

---

## What Phiber Needs to Bridge from Zed

### Required Bridges

```rust
// 1. READ current user
let user = user_store.read(cx).current_user();
// -> Option<Arc<User>>

// 2. READ connection status
let status = client.status();
let status = &*status.borrow();
// -> &client::Status

// 3. READ organizations
let orgs = user_store.read(cx).organizations();
// -> &[Arc<Organization>]

// 4. READ plan for org
let plan = user_store.read(cx).plan_for_organization(&org.id);
// -> Option<Plan>  (Plan: Free, Pro, etc.)

// 5. READ notifications
let notif_count = NotificationStore::global(cx)
    .read(cx).unread_notification_count();

// 6. ACTIONS (dispatch via cx)
cx.dispatch_action(zed_actions::OpenSettings.boxed_clone());
cx.dispatch_action(Box::new(zed_actions::OpenKeymap));
cx.dispatch_action(client::SignOut.boxed_clone());
cx.dispatch_action(zed_actions::theme_selector::Toggle::default().boxed_clone());
```

### What Phiber Adds (Not in Zed)

```
- Custom status message (text + emoji)  <- purely Phiber custom state
- 4-state online status (Online/Idle/DND/Invisible)  <- Phiber custom
- Activity feed (local, not from server)  <- Phiber custom
```

---

## Store Data Model (Updated)

### store/modals/profile/def.rs — additions needed

```rust
// Connection status display (maps from client::Status)
pub enum BConnectionStatus {
    Connected,      // green  - Status::Connected
    Connecting,     // amber  - Status::Connecting, Authenticating, Reconnecting
    Disconnected,   // grey   - Status::SignedOut
    Error,          // red    - Status::AuthenticationError, ConnectionError
    SignedOut,      // grey/x - Status::SignedOut, UpgradeRequired
}

// Presence status (user-settable, Phiber custom)
pub enum BPresenceStatus {
    Online,     // green dot
    Idle,       // amber dot
    Dnd,        // red dot (Do Not Disturb)
    Invisible,  // grey dot
}

impl BPresenceStatus {
    pub fn label(self) -> &'static str;
    pub fn icon(self) -> &'static str;  // SVG asset key
}
```

---

## Collab Panel Notes (Future)

Zed's `collab_panel.rs` (169KB!) has a full Discord-like panel:
- Online contacts with busy/online indicators
- Channels (Discord-like channel list)
- Active call participants with audio status
- Contact request management (accept/reject)
- Channel creation and membership

Phiber can integrate this in the future as a sidebar `BPartType::Collab`.
For now the profile modal is the primary account touch point.
