pub const STYLES: &str = r#"
@import url('https://fonts.googleapis.com/css2?family=Noto+Sans+TC:wght@300;400;500;700;900&family=Space+Grotesk:wght@400;600;700&display=swap');
@import url('https://fonts.googleapis.com/css2?family=Material+Symbols+Outlined&display=block');

:root {
  --md-source: #279b57;

  /* ── Primary tonal palette ── */
  --md-ref-palette-primary0:   #000000;
  --md-ref-palette-primary10:  #002110;
  --md-ref-palette-primary20:  #003920;
  --md-ref-palette-primary30:  #005232;
  --md-ref-palette-primary40:  #006c45;
  --md-ref-palette-primary50:  #008757;
  --md-ref-palette-primary60:  #279b57;
  --md-ref-palette-primary70:  #4eb674;
  --md-ref-palette-primary80:  #6dd391;
  --md-ref-palette-primary90:  #8af0ad;
  --md-ref-palette-primary95:  #a9fec7;
  --md-ref-palette-primary99:  #f5fff4;
  --md-ref-palette-primary100: #ffffff;

  /* ── Secondary tonal palette ── */
  --md-ref-palette-secondary0:   #000000;
  --md-ref-palette-secondary10:  #0b1f12;
  --md-ref-palette-secondary20:  #203527;
  --md-ref-palette-secondary30:  #364b3c;
  --md-ref-palette-secondary40:  #4e6353;
  --md-ref-palette-secondary50:  #667b6a;
  --md-ref-palette-secondary60:  #7c9480;
  --md-ref-palette-secondary70:  #96af9a;
  --md-ref-palette-secondary80:  #b1cbb4;
  --md-ref-palette-secondary90:  #cce7cf;
  --md-ref-palette-secondary95:  #daf5dd;
  --md-ref-palette-secondary99:  #f4ffee;
  --md-ref-palette-secondary100: #ffffff;

  /* ── Tertiary tonal palette ── */
  --md-ref-palette-tertiary0:   #000000;
  --md-ref-palette-tertiary10:  #001f26;
  --md-ref-palette-tertiary20:  #003641;
  --md-ref-palette-tertiary30:  #004d5b;
  --md-ref-palette-tertiary40:  #2d5f6d;
  --md-ref-palette-tertiary50:  #467886;
  --md-ref-palette-tertiary60:  #5f92a1;
  --md-ref-palette-tertiary70:  #79adbc;
  --md-ref-palette-tertiary80:  #94c8d8;
  --md-ref-palette-tertiary90:  #b0e4f5;
  --md-ref-palette-tertiary95:  #c5f3ff;
  --md-ref-palette-tertiary99:  #f7fdff;
  --md-ref-palette-tertiary100: #ffffff;

  /* ── Error tonal palette ── */
  --md-ref-palette-error0:   #000000;
  --md-ref-palette-error10:  #410002;
  --md-ref-palette-error20:  #690005;
  --md-ref-palette-error30:  #93000a;
  --md-ref-palette-error40:  #ba1a1a;
  --md-ref-palette-error50:  #de3730;
  --md-ref-palette-error60:  #ff5449;
  --md-ref-palette-error70:  #ff897d;
  --md-ref-palette-error80:  #ffb4ab;
  --md-ref-palette-error90:  #ffdad6;
  --md-ref-palette-error95:  #ffedea;
  --md-ref-palette-error99:  #fff8f7;
  --md-ref-palette-error100: #ffffff;

  /* ── Neutral tonal palette ── */
  --md-ref-palette-neutral0:   #000000;
  --md-ref-palette-neutral10:  #1a1c1a;
  --md-ref-palette-neutral20:  #2f312e;
  --md-ref-palette-neutral30:  #464745;
  --md-ref-palette-neutral40:  #5d5f5c;
  --md-ref-palette-neutral50:  #767774;
  --md-ref-palette-neutral60:  #777976;
  --md-ref-palette-neutral70:  #91938f;
  --md-ref-palette-neutral80:  #c6c8c3;
  --md-ref-palette-neutral90:  #e2e4df;
  --md-ref-palette-neutral95:  #f0f2ed;
  --md-ref-palette-neutral99:  #fdfef9;
  --md-ref-palette-neutral100: #ffffff;

  /* ── Neutral Variant tonal palette ── */
  --md-ref-palette-neutral-variant0:   #000000;
  --md-ref-palette-neutral-variant10:  #191d1b;
  --md-ref-palette-neutral-variant20:  #2e3230;
  --md-ref-palette-neutral-variant30:  #444946;
  --md-ref-palette-neutral-variant40:  #5c605d;
  --md-ref-palette-neutral-variant50:  #747975;
  --md-ref-palette-neutral-variant60:  #757974;
  --md-ref-palette-neutral-variant70:  #8e928d;
  --md-ref-palette-neutral-variant80:  #c1c9bf;
  --md-ref-palette-neutral-variant90:  #dde5da;
  --md-ref-palette-neutral-variant95:  #ebf3e8;
  --md-ref-palette-neutral-variant99:  #f6fef3;
  --md-ref-palette-neutral-variant100: #ffffff;

  /* ── MD3 Color Roles (Light) ── */
  --md-sys-color-primary:            var(--md-ref-palette-primary40);
  --md-sys-color-on-primary:         #ffffff;
  --md-sys-color-primary-container:  var(--md-ref-palette-primary90);
  --md-sys-color-on-primary-container: var(--md-ref-palette-primary10);

  --md-sys-color-secondary:            var(--md-ref-palette-secondary40);
  --md-sys-color-on-secondary:         #ffffff;
  --md-sys-color-secondary-container:  var(--md-ref-palette-secondary90);
  --md-sys-color-on-secondary-container: var(--md-ref-palette-secondary10);

  --md-sys-color-tertiary:            var(--md-ref-palette-tertiary40);
  --md-sys-color-on-tertiary:         #ffffff;
  --md-sys-color-tertiary-container:  var(--md-ref-palette-tertiary90);
  --md-sys-color-on-tertiary-container: var(--md-ref-palette-tertiary10);

  --md-sys-color-error:            var(--md-ref-palette-error40);
  --md-sys-color-on-error:         #ffffff;
  --md-sys-color-error-container:  var(--md-ref-palette-error90);
  --md-sys-color-on-error-container: var(--md-ref-palette-error10);

  --md-sys-color-surface-dim:                  #d8dad5;
  --md-sys-color-surface:                      #fdfef9;
  --md-sys-color-surface-bright:               #fdfef9;
  --md-sys-color-surface-container-lowest:     #ffffff;
  --md-sys-color-surface-container-low:        #f7f8f3;
  --md-sys-color-surface-container:            #f0f2ed;
  --md-sys-color-surface-container-high:       #ebece7;
  --md-sys-color-surface-container-highest:    #e5e6e2;
  --md-sys-color-on-surface:                   var(--md-ref-palette-neutral10);
  --md-sys-color-on-surface-variant:           var(--md-ref-palette-neutral-variant30);
  --md-sys-color-outline:                      var(--md-ref-palette-neutral-variant60);
  --md-sys-color-outline-variant:              var(--md-ref-palette-neutral-variant80);

  --md-sys-color-inverse-surface:     var(--md-ref-palette-neutral20);
  --md-sys-color-inverse-on-surface:  var(--md-ref-palette-neutral95);
  --md-sys-color-inverse-primary:     var(--md-ref-palette-primary80);

  --md-sys-color-surface-tint: var(--md-ref-palette-primary40);
  --md-sys-color-background:      var(--md-sys-color-surface);
  --md-sys-color-on-background:   var(--md-sys-color-on-surface);

  --md-sys-elevation-level0: none;
  --md-sys-elevation-level1: 0 1px 2px rgba(0,0,0,.3), 0 1px 3px 1px rgba(0,0,0,.15);
  --md-sys-elevation-level2: 0 1px 2px rgba(0,0,0,.3), 0 2px 6px 2px rgba(0,0,0,.15);
  --md-sys-elevation-level3: 0 1px 3px rgba(0,0,0,.3), 0 4px 8px 3px rgba(0,0,0,.15);
  --md-sys-elevation-level4: 0 2px 3px rgba(0,0,0,.3), 0 6px 10px 4px rgba(0,0,0,.15);
  --md-sys-elevation-level5: 0 4px 4px rgba(0,0,0,.3), 0 8px 12px 6px rgba(0,0,0,.15);

  --md-sys-shape-corner-extra-small: 4px;
  --md-sys-shape-corner-small:       8px;
  --md-sys-shape-corner-medium:      12px;
  --md-sys-shape-corner-large:       16px;
  --md-sys-shape-corner-extra-large: 24px;
  --md-sys-shape-corner-full:        9999px;

  --safe-bottom: env(safe-area-inset-bottom, 0px);

  --elev-1: var(--md-sys-elevation-level1);
  --elev-2: var(--md-sys-elevation-level2);
  --elev-3: var(--md-sys-elevation-level3);

  --shape-xs:   var(--md-sys-shape-corner-extra-small);
  --shape-sm:   var(--md-sys-shape-corner-small);
  --shape-md:   var(--md-sys-shape-corner-medium);
  --shape-lg:   var(--md-sys-shape-corner-large);
  --shape-xl:   var(--md-sys-shape-corner-extra-large);
  --shape-xxl:  28px;
  --shape-full: var(--md-sys-shape-corner-full);

  /* ── Motion tokens (MD3) ── */
  --md-sys-motion-easing-emphasized: cubic-bezier(0.2, 0, 0, 1);
  --md-sys-motion-easing-emphasized-accelerate: cubic-bezier(0.3, 0, 0.8, 0.15);
  --md-sys-motion-easing-emphasized-decelerate: cubic-bezier(0.05, 0.7, 0.1, 1);
  --md-sys-motion-easing-standard: cubic-bezier(0.2, 0, 0, 1);
  --md-sys-motion-easing-standard-decelerate: cubic-bezier(0, 0, 0, 1);
  --md-sys-motion-duration-short4: 200ms;
  --md-sys-motion-duration-medium2: 300ms;
  --md-sys-motion-duration-medium4: 400ms;
  --md-sys-motion-duration-long1: 450ms;
  --md-sys-motion-duration-long3: 550ms;
}

:root[data-theme="dark"] {
  --md-sys-color-primary: var(--md-ref-palette-primary80);
  --md-sys-color-on-primary: var(--md-ref-palette-primary20);
  --md-sys-color-primary-container: var(--md-ref-palette-primary30);
  --md-sys-color-on-primary-container: var(--md-ref-palette-primary90);
  --md-sys-color-secondary: var(--md-ref-palette-secondary80);
  --md-sys-color-on-secondary: var(--md-ref-palette-secondary20);
  --md-sys-color-secondary-container: var(--md-ref-palette-secondary30);
  --md-sys-color-on-secondary-container: var(--md-ref-palette-secondary90);
  --md-sys-color-tertiary: var(--md-ref-palette-tertiary80);
  --md-sys-color-on-tertiary: var(--md-ref-palette-tertiary20);
  --md-sys-color-tertiary-container: var(--md-ref-palette-tertiary30);
  --md-sys-color-on-tertiary-container: var(--md-ref-palette-tertiary90);
  --md-sys-color-error: var(--md-ref-palette-error80);
  --md-sys-color-on-error: var(--md-ref-palette-error20);
  --md-sys-color-error-container: var(--md-ref-palette-error30);
  --md-sys-color-on-error-container: var(--md-ref-palette-error90);
  --md-sys-color-surface: var(--md-ref-palette-neutral10);
  --md-sys-color-surface-dim: #131513;
  --md-sys-color-surface-bright: #313331;
  --md-sys-color-surface-container-lowest: #000000;
  --md-sys-color-surface-container-low: #1a1c1a;
  --md-sys-color-surface-container: #222422;
  --md-sys-color-surface-container-high: #2c2e2c;
  --md-sys-color-surface-container-highest: #333533;
  --md-sys-color-on-surface: var(--md-ref-palette-neutral90);
  --md-sys-color-on-surface-variant: var(--md-ref-palette-neutral-variant80);
  --md-sys-color-outline: var(--md-ref-palette-neutral-variant60);
  --md-sys-color-outline-variant: var(--md-ref-palette-neutral-variant40);
  --md-sys-color-inverse-surface: var(--md-ref-palette-neutral90);
  --md-sys-color-inverse-on-surface: var(--md-ref-palette-neutral20);
  --md-sys-color-surface-tint: var(--md-ref-palette-primary80);
}

*,*::before,*::after { box-sizing: border-box; margin: 0; padding: 0; }
html { height: 100%; overflow: hidden; }
#main { width: 100%; height: 100%; }

input, textarea {
    -webkit-user-select: text !important;
    user-select: text !important;
    -webkit-touch-callout: default !important;
}

body {
    font-family: 'Noto Sans TC','Space Grotesk',sans-serif;
    background: var(--md-sys-color-surface);
    color: var(--md-sys-color-on-surface);
    height: 100dvh; min-height: 100dvh;
    display: flex; justify-content: center; align-items: center;
    overflow: hidden; position: relative;
    -webkit-tap-highlight-color: transparent;
    transition: background-color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized), color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized);
}
body::before,body::after {
    content: ''; position: fixed; border-radius: 50%;
    pointer-events: none; z-index: 0;
}
body::before {
    top: -40%; right: -25%; width: 50vw; height: 50vw;
    background: radial-gradient(circle,rgba(39,155,87,.07) 0%,transparent 65%);
}
body::after {
    bottom: -35%; left: -15%; width: 40vw; height: 40vw;
    background: radial-gradient(circle,rgba(39,155,87,.05) 0%,transparent 65%);
}

@keyframes fadeUp { from { opacity:0; transform:translateY(28px); } to { opacity:1; transform:translateY(0); } }

/* ── Material Symbols ── */
.material-symbols-outlined {
  font-family: 'Material Symbols Outlined';
  font-weight: normal;
  font-style: normal;
  font-size: 1.2em;
  line-height: 1;
  letter-spacing: normal;
  text-transform: none;
  display: inline-block;
  white-space: nowrap;
  word-wrap: normal;
  direction: ltr;
  vertical-align: middle;
  -webkit-font-feature-settings: 'liga';
  -webkit-font-smoothing: antialiased;
}

.upload-wrapper {
    display: flex; flex-direction: column; justify-content: safe center; gap: 16px;
    width: 100%; max-width: 640px; position: relative; z-index: 1;
    min-height: 90dvh; max-height: 92dvh; overflow-y: auto;
    -webkit-overflow-scrolling: touch; scrollbar-width: thin; min-width: 0;
}
.upload-wrapper::-webkit-scrollbar { width: 6px; }
.upload-wrapper::-webkit-scrollbar-track { background: transparent; }
.upload-wrapper::-webkit-scrollbar-thumb { background: var(--md-sys-color-outline-variant); border-radius: 4px; }

.upload-container {
    background: var(--md-sys-color-surface-container);
    padding: clamp(40px, 8vh, 80px) clamp(20px,4vw,40px);
    border-radius: var(--shape-xxl);
    width: 100%; max-width: 640px; text-align: center; position: relative; z-index: 1;
    display: flex; flex-direction: column; justify-content: flex-start;
    animation: fadeUp var(--md-sys-motion-duration-long3) var(--md-sys-motion-easing-emphasized-decelerate);
    transition: background-color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized), color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized); min-width: 0;
}
.upload-container h2 {
    font-size: clamp(1.4em,2.5vw,1.8em); font-weight: 700; margin-bottom: 8px;
}
.upload-container .subtitle {
    color: var(--md-sys-color-on-surface-variant);
    font-size: clamp(.85em,1.6vw,.95em); line-height: 1.7;
    margin-bottom: clamp(20px,4vw,32px);
}
.upload-container .subtitle strong { color: var(--md-sys-color-primary); font-weight: 600; }

.url-textarea {
    width: 100%; min-height: 120px; padding: 15px;
    background: transparent;
    border: 2px solid var(--md-sys-color-outline);
    border-radius: var(--shape-xs);
    color: var(--md-sys-color-on-surface);
    font-size: 1rem; font-family: inherit; outline: none;
    transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), padding var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    -webkit-user-select: text;
    user-select: text;
    resize: vertical; line-height: 1.5; margin-top: 12px;
    overflow-y: hidden;
}
.url-textarea:focus {
    border-color: var(--md-sys-color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent);
}
.url-textarea::placeholder {
    color: var(--md-sys-color-on-surface-variant); opacity: 0.7;
}

.go-btn, .text-btn {
    padding: 10px 24px; min-height: 48px; border: none;
    border-radius: var(--shape-full);
    font-size: .95em; font-weight: 500; cursor: pointer;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), transform .1s;
    display: inline-flex; align-items: center; justify-content: center;
    white-space: nowrap; font-family: inherit; width: 100%;
}
.go-btn { background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); }
.text-btn { background: transparent; color: var(--md-sys-color-primary); }
.action-btns-row { display: flex; flex-direction: column; gap: 12px; margin-top: 24px; }

@media (hover: hover) {
    .go-btn:hover:not(.inert), .text-btn:hover:not(.inert) { box-shadow: var(--elev-1); }
    .text-btn:hover:not(.inert) { background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent); }
}
.go-btn:active:not(.inert), .text-btn:active:not(.inert) { box-shadow: none; transform: scale(0.97); }
.go-btn.inert, .text-btn.inert {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent) !important;
    color: color-mix(in srgb, var(--md-sys-color-on-surface) 38%, transparent) !important;
    cursor: not-allowed; box-shadow: none;
}

.fetch-error {
    color: var(--md-sys-color-error); font-size: .85em; margin-top: 12px; font-weight: 500;
    white-space: pre-wrap; word-break: break-word; text-align: left;
}

.html-fallback {
    margin-top: 20px; padding: 20px;
    background: var(--md-sys-color-surface-container-lowest);
    border: 1px solid var(--md-sys-color-outline-variant);
    border-radius: var(--shape-lg);
    text-align: left;     animation: fadeUp var(--md-sys-motion-duration-medium3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.fallback-title {
    font-size: 1em; font-weight: 600; color: var(--md-sys-color-on-surface);
    margin-bottom: 8px;
}
.fallback-hint {
    color: var(--md-sys-color-on-surface-variant);
    font-size: .85em; line-height: 1.6; white-space: pre-line;
    margin-bottom: 16px;
}
.fallback-url-input {
    width: 100%; padding: 12px 15px; margin-bottom: 12px;
    background: transparent;
    border: 2px solid var(--md-sys-color-outline);
    border-radius: var(--shape-xs);
    color: var(--md-sys-color-on-surface);
    font-size: .95rem; font-family: inherit; outline: none;
    transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    -webkit-user-select: text;
    user-select: text;
}
.fallback-url-input:focus {
    border-color: var(--md-sys-color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent);
}
.fallback-url-input::placeholder { color: var(--md-sys-color-on-surface-variant); opacity: 0.7; }
.open-page-btn {
    padding: 10px 20px; min-height: 44px; margin-bottom: 12px;
    border: 1px solid var(--md-sys-color-outline); background: transparent;
    color: var(--md-sys-color-primary); border-radius: var(--shape-full);
    font-size: .9em; font-weight: 500; cursor: pointer; font-family: inherit;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), transform .1s; width: 100%;
    display: inline-flex; align-items: center; justify-content: center;
}
@media (hover: hover) {
    .open-page-btn:hover:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent); }
}
.open-page-btn:active:not(:disabled) { transform: scale(0.97); }
.open-page-btn:disabled { opacity: 0.5; cursor: not-allowed; }
.html-textarea {
    width: 100%; min-height: 220px; padding: 15px;
    background: transparent;
    border: 2px solid var(--md-sys-color-outline);
    border-radius: var(--shape-xs);
    color: var(--md-sys-color-on-surface);
    font-size: .85rem; font-family: ui-monospace, SFMono-Regular, monospace;
    line-height: 1.5; resize: vertical; outline: none;
    transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    -webkit-user-select: text;
    user-select: text;
}
.html-textarea:focus {
    border-color: var(--md-sys-color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent);
}
.html-textarea::placeholder { color: var(--md-sys-color-on-surface-variant); opacity: 0.7; }
.fallback-actions { display: flex; gap: 12px; margin-top: 16px; }
.fallback-actions .go-btn, .fallback-actions .text-btn { flex: 1; width: auto; }

.history-card {
    background: var(--md-sys-color-surface-container);
    padding: clamp(20px,3vw,32px) clamp(24px,4vw,40px);
    border-radius: var(--shape-xxl);
    text-align: left; flex-shrink: 0; max-height: 28vh;
    display: flex; flex-direction: column; overflow: hidden;
    animation: fadeUp var(--md-sys-motion-duration-long3) var(--md-sys-motion-easing-emphasized-decelerate) .1s both;
    transition: background-color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized), color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized); min-width: 0;
}
.history-card h3 {
    font-size: 0.9em; color: var(--md-sys-color-on-surface-variant);
    margin-bottom: 12px; font-weight: 600;
}

.history-list {
    display: flex; flex-direction: column; gap: 8px; overflow-y: auto; padding-right: 4px;
    -webkit-overflow-scrolling: touch; scrollbar-width: thin; min-width: 0;
}
.history-list::-webkit-scrollbar { width: 4px; }
.history-list::-webkit-scrollbar-track { background: transparent; }
.history-list::-webkit-scrollbar-thumb { background: var(--md-sys-color-outline-variant); border-radius: 4px; }

button.history-item {
    width: 100%; background: var(--md-sys-color-surface-container-highest); border: none;
    padding: 12px 16px; border-radius: var(--shape-sm); font-size: 0.85em; font-family: inherit;
    color: var(--md-sys-color-on-surface); cursor: pointer; white-space: nowrap; text-align: left;
    display: flex; align-items: center; gap: 10px; justify-content: space-between;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard); min-width: 0;
}
button.history-item:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, var(--md-sys-color-surface-container-highest)); }
button.history-item:active { transform: scale(0.98); }
.hist-icon { flex-shrink: 0; opacity: 0.6; font-size: 0.9em; }
.hist-text {
    flex: 1; overflow-x: auto; white-space: nowrap; -ms-overflow-style: none; scrollbar-width: none;
    mask-image: linear-gradient(to right, #000 85%, transparent 100%);
    -webkit-mask-image: linear-gradient(to right, #000 85%, transparent 100%);
}
.hist-text::-webkit-scrollbar { display: none; }
.hist-arrow { flex-shrink: 0; opacity: 0.5; font-size: 1.2em; }

.quiz-container {
    background: var(--md-sys-color-surface-container);
    padding: clamp(24px,4vw,40px) clamp(20px,4vw,48px) clamp(24px,4vw,36px);
    border-radius: var(--shape-xxl);
    width: 100%; max-width: 700px;
    text-align: center; position: relative; z-index: 1;
    animation: fadeUp var(--md-sys-motion-duration-medium4) var(--md-sys-motion-easing-emphasized-decelerate); max-height: 92dvh; overflow-y: auto;
    transition: background-color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized), color var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized);
    -webkit-user-select: none;
    user-select: none;
}
.quiz-container:focus { outline: none; }
.quiz-container:focus-visible { outline: 2px solid var(--md-sys-color-primary); outline-offset: 4px; }

.status-bar {
    display: flex; justify-content: space-between; align-items: center;
    margin-bottom: 12px; flex-wrap: wrap; gap: 8px;
}
.status-left { display: flex; align-items: center; gap: 12px; }
.status-text {
    color: var(--md-sys-color-on-surface-variant);
    font-size: clamp(.85em,1.5vw,1em); font-weight: 500;
}

.badge {
    display: inline-flex; align-items: center; gap: 6px;
    font-size: clamp(.75em,1.2vw,.85em); font-weight: 500;
    padding: 6px 12px; border-radius: var(--shape-full);
    border: 1px solid var(--md-sys-color-outline);
    background: transparent; color: var(--md-sys-color-on-surface-variant);
}
.badge-skip { border-color: transparent; background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); }
.badge-review { border-color: transparent; background: var(--md-sys-color-tertiary-container); color: var(--md-sys-color-on-tertiary-container); }

#question-word {
    font-family: 'Space Grotesk','Noto Sans TC',sans-serif;
    font-size: clamp(1.8em,4.5vw,2.8em); font-weight: 700;
    color: var(--md-sys-color-on-surface);
    margin: 16px 0 clamp(24px,4vw,40px); letter-spacing: -.5px;
    word-break: break-word; line-height: 1.3; min-height: 4em;
    transition: opacity var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.ans-en { color: var(--md-sys-color-primary); font-weight: 700; }
.ans-zh {
    color: var(--md-sys-color-on-surface-variant);
    font-size: .5em; display: block; margin-top: 12px; font-weight: 500;
}

.options-container { display: flex; flex-direction: column; gap: clamp(10px,2vw,16px); }
.option-btn {
    display: flex; align-items: center; gap: clamp(12px,2vw,16px); width: 100%;
    padding: clamp(14px,2.5vw,20px);
    background: transparent;
    border: 1px solid var(--md-sys-color-outline-variant);
    border-radius: var(--shape-lg);
    font-size: clamp(.95em,1.8vw,1.1em);
    font-family: inherit; cursor: pointer; text-align: left; line-height: 1.5;
    white-space: pre-line; color: var(--md-sys-color-on-surface);
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    -webkit-tap-highlight-color: transparent;
}
.option-btn:focus { outline: none; }
.option-btn:disabled { opacity: 1; cursor: default; }

.option-btn .opt-label {
    display: inline-flex; align-items: center; justify-content: center;
    width: clamp(28px,4.5vw,36px); height: clamp(28px,4.5vw,36px);
    border-radius: 50%; background: var(--md-sys-color-surface-container-highest);
    font-size: .85em; font-weight: 500;
    color: var(--md-sys-color-on-surface-variant); flex-shrink: 0;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .option-btn:hover:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}
.option-btn:active:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent); }

.option-btn.correct {
    background: var(--md-sys-color-primary-container); border-color: transparent;
    color: var(--md-sys-color-on-primary-container);
}
.option-btn.correct .opt-label { background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); }
.option-btn.wrong {
    background: var(--md-sys-color-error-container); border-color: transparent;
    color: var(--md-sys-color-on-error-container);
}
.option-btn.wrong .opt-label { background: var(--md-sys-color-error); color: var(--md-sys-color-on-error); }
.option-btn.dimmed { opacity: .5; }
.option-btn .opt-text { display: flex; flex-direction: column; }
.option-btn .opt-pair {
    display: block; margin-top: 4px;
    font-size: .75em; color: var(--md-sys-color-on-surface-variant); font-weight: 500;
}

.controls {
    display: flex; justify-content: center; gap: clamp(8px,1.5vw,16px);
    margin-top: clamp(24px,4vw,36px); flex-wrap: wrap;
}

.ctrl-btn {
    padding: clamp(10px,1.8vw,12px) clamp(20px,3vw,28px);
    min-height: 40px;
    font-size: clamp(.9em,1.5vw,1em); font-family: inherit; font-weight: 500;
    border-radius: var(--shape-full);
    display: inline-flex; align-items: center; justify-content: center;
    cursor: pointer; transition: all var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.ctrl-btn:focus { outline: none; }
.ctrl-btn:disabled {
    background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent) !important;
    color: color-mix(in srgb, var(--md-sys-color-on-surface) 38%, transparent) !important;
    border-color: transparent !important; cursor: not-allowed; box-shadow: none !important;
}
.ctrl-btn:active:not(:disabled) { transform: scale(0.97); }

.ctrl-btn.outlined { border: 1px solid var(--md-sys-color-outline); background: transparent; color: var(--md-sys-color-primary); }
@media (hover: hover) {
    .ctrl-btn.outlined:hover:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent); border-color: var(--md-sys-color-primary); }
}

.ctrl-btn.tonal { border: none; background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); }
@media (hover: hover) {
    .ctrl-btn.tonal:hover:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-on-secondary-container) 8%, var(--md-sys-color-secondary-container)); }
}

.ctrl-btn.filled { border: none; background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); }
@media (hover: hover) {
    .ctrl-btn.filled:hover:not(:disabled) { background: color-mix(in srgb, var(--md-sys-color-on-primary) 8%, var(--md-sys-color-primary)); box-shadow: var(--elev-1); }
}

.back-bar {
    display: flex; align-items: center; gap: 12px; margin-bottom: 12px;
}
.q-correct { color: var(--md-sys-color-primary); font-weight: 700; font-size: 1em; }
.q-wrong { color: var(--md-sys-color-error); font-weight: 700; font-size: 1em; }
.q-plus { color: var(--md-sys-color-on-surface); font-weight: 400; font-size: 1em; margin: 0 2px; }

/* ── Unified top-left icon buttons (pause/settings) ── */
.top-icon-btn {
    position: absolute; top: 20px;
    width: 44px; height: 44px; border: none; border-radius: 50%;
    background: transparent;
    color: var(--md-sys-color-on-surface-variant);
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    z-index: 10; font-size: 1.3em;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), transform .1s;
}
.top-icon-btn:active { transform: scale(0.92); }
@media (hover: hover) {
    .top-icon-btn:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}

/* ── Pause overlay ── */
.pause-overlay {
    position: fixed; inset: 0; z-index: 1000;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    animation: fadeUp var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.pause-dialog {
    background: var(--md-sys-color-surface-container);
    border-radius: var(--shape-xxl);
    padding: clamp(24px,5vw,32px);
    text-align: center; max-width: 320px; width: 85%;
    box-shadow: var(--elev-3);
}
.pause-title {
    font-size: 1.15em; font-weight: 600; margin-bottom: 20px;
    color: var(--md-sys-color-on-surface);
}
.pause-btn-row {
    display: flex; gap: clamp(16px,3vw,24px); justify-content: center;
}
.pause-icon-box {
    display: flex; flex-direction: column; align-items: center; gap: 10px;
    padding: 18px 28px; border: 2px solid var(--md-sys-color-outline-variant);
    border-radius: var(--shape-xl);
    cursor: pointer; background: transparent; font-family: inherit;
    color: var(--md-sys-color-on-surface);
    transition: background var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
}
.pause-icon-box:active { transform: scale(0.97); }
@media (hover: hover) {
    .pause-icon-box:hover { background: var(--md-sys-color-surface-container-high); }
}
.pause-icon-box span.material-symbols-outlined {
    font-size: 2em; color: var(--md-sys-color-primary);
}
.pause-btn-label { font-size: .9em; font-weight: 500; }

/* ── Settings overlay ── */
.settings-overlay {
    position: fixed; inset: 0; z-index: 500;
    background: var(--md-sys-color-surface);
    display: flex; flex-direction: column;
    animation: fadeUp var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-emphasized-decelerate);
}
.settings-topbar {
    display: flex; align-items: center; gap: 4px; padding: 8px 4px;
    min-height: 64px;
    font-size: 1.2em; font-weight: 500;
}
.settings-topbar-title {
    flex: 1; font-size: 28px; font-weight: 400; padding-left: 4px;
}
.settings-topbar-btn {
    width: 40px; height: 40px; border: none; border-radius: 50%;
    background: transparent; color: var(--md-sys-color-on-surface-variant);
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-size: 1.2em; transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .settings-topbar-btn:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}
.settings-topbar-btn:active { background: color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent); }
.settings-close {
    width: 40px; height: 40px; border: none; border-radius: 50%;
    background: transparent; color: var(--md-sys-color-on-surface);
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-size: 1.2em; transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .settings-close:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}
.settings-body {
    padding: 16px 24px; flex: 1; overflow-y: auto;
}
.settings-item {
    display: flex; align-items: center; gap: 16px;
    padding: 16px 12px; border-radius: var(--shape-medium);
    cursor: pointer; transition: background var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .settings-item:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent); }
}
.settings-item-icon {
    width: 40px; height: 40px; border-radius: var(--shape-full);
    display: flex; align-items: center; justify-content: center;
    font-size: 1.3em;
    color: var(--md-sys-color-on-surface-variant);
    background: var(--md-sys-color-surface-container-high);
}
.settings-item-label {
    flex: 1; font-size: 1em; font-weight: 500;
    color: var(--md-sys-color-on-surface);
}
/* MD3-style switch */
.settings-switch {
    position: relative; width: 52px; height: 32px; flex-shrink: 0;
    background: var(--md-sys-color-surface-container-highest);
    border: 2px solid var(--md-sys-color-outline);
    border-radius: 9999px; cursor: pointer;     transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    padding: 0;
}
.settings-switch.on {
    background: var(--md-sys-color-primary); border-color: var(--md-sys-color-primary);
}
.settings-switch::after {
    content: ''; position: absolute; top: 2px; left: 2px;
    width: 24px; height: 24px; border-radius: 50%;
    background: var(--md-sys-color-outline);
    transition: transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard), background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.settings-switch.on::after {
    transform: translateX(20px);
    background: var(--md-sys-color-on-primary);
}

/* MD3 segmented button for theme mode selector */
.settings-section-label {
    font-size: .75em; font-weight: 500; text-transform: uppercase; letter-spacing: .1em;
    color: var(--md-sys-color-on-surface-variant);
    padding: 4px 8px; margin-bottom: 12px;
}
.theme-segmented {
    display: flex; border: 1px solid var(--md-sys-color-outline);
    border-radius: var(--shape-full); overflow: hidden;
    margin-bottom: 12px;
}
.theme-btn {
    flex: 1; display: flex; align-items: center; justify-content: center; gap: 6px;
    padding: 10px 8px; border: none; cursor: pointer;
    font-size: .85em; font-weight: 500; font-family: inherit;
    background: transparent; color: var(--md-sys-color-on-surface-variant);
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
                color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.theme-btn:not(:last-child) {
    border-right: 1px solid var(--md-sys-color-outline);
}
.theme-btn.active {
    background: var(--md-sys-color-secondary-container);
    color: var(--md-sys-color-on-secondary-container);
}
.theme-btn .material-symbols-outlined {
    font-size: 1.2em; font-variation-settings: 'FILL' 1;
}
@media (hover: hover) {
    .theme-btn:not(.active):hover {
        background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent);
    }
}

/* ── Version label in settings body ── */
.settings-version {
    text-align: right;
    padding: 8px 24px 12px;
    font-size: 12px;
    color: var(--md-sys-color-on-surface-variant);
}

/* ── License dialog ── */
.license-overlay {
    position: fixed; inset: 0; z-index: 1000;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    animation: fadeUp var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.license-dialog {
    background: var(--md-sys-color-surface-container);
    border-radius: var(--shape-xxl);
    width: 85%; max-width: 480px; max-height: 80vh;
    display: flex; flex-direction: column;
    box-shadow: var(--elev-3); overflow: hidden;
}
.license-dialog-topbar {
    display: flex; align-items: center; gap: 4px;
    padding: 8px 4px; min-height: 56px;
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
    flex-shrink: 0;
}
.license-dialog-title {
    flex: 1; font-size: 1.1em; font-weight: 600;
    color: var(--md-sys-color-on-surface); padding-left: 8px;
}
.license-dialog-close {
    width: 40px; height: 40px; border: none; border-radius: 50%;
    background: transparent; color: var(--md-sys-color-on-surface);
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-size: 1.2em; transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .license-dialog-close:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}
.license-list {
    flex: 1; overflow-y: auto; padding: 8px; -webkit-overflow-scrolling: touch;
}
.license-list::-webkit-scrollbar { width: 4px; }
.license-list::-webkit-scrollbar-track { background: transparent; }
.license-list::-webkit-scrollbar-thumb { background: var(--md-sys-color-outline-variant); border-radius: 4px; }
.license-item {
    display: flex; align-items: center; gap: 12px;
    width: 100%; padding: 12px 12px; border: none; border-radius: var(--shape-sm);
    background: transparent; cursor: pointer; font-family: inherit;
    color: var(--md-sys-color-on-surface);
    transition: background var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard); text-align: left;
}
@media (hover: hover) {
    .license-item:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 4%, transparent); }
}
.license-item-name {
    flex: 1; font-size: .95em; font-weight: 500;
    min-width: 0; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;
}
.license-item-type {
    flex-shrink: 0; font-size: .75em; font-weight: 500;
    padding: 4px 10px; border-radius: var(--shape-full);
    background: var(--md-sys-color-surface-container-high);
    color: var(--md-sys-color-on-surface-variant);
}

/* ── License detail ── */
.license-detail-overlay {
    position: fixed; inset: 0; z-index: 1100;
    background: rgba(0,0,0,0.4);
    display: flex; align-items: center; justify-content: center;
    animation: fadeUp var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.license-detail-dialog {
    background: var(--md-sys-color-surface-container);
    border-radius: var(--shape-xxl);
    width: 85%; max-width: 560px; max-height: 80vh;
    display: flex; flex-direction: column;
    box-shadow: var(--elev-3); overflow: hidden;
}
.license-detail-topbar {
    display: flex; align-items: center; gap: 4px;
    padding: 8px 4px; min-height: 56px;
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
    flex-shrink: 0;
}
.license-detail-title {
    flex: 1; font-size: 1.1em; font-weight: 600;
    color: var(--md-sys-color-on-surface); padding-left: 8px;
}
.license-detail-close {
    width: 40px; height: 40px; border: none; border-radius: 50%;
    background: transparent; color: var(--md-sys-color-on-surface);
    cursor: pointer; display: flex; align-items: center; justify-content: center;
    font-size: 1.2em; transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .license-detail-close:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 8%, transparent); }
}
.license-detail-body {
    flex: 1; overflow-y: auto; padding: 20px 16px;
    font-size: .8em; line-height: 1.6;
    white-space: pre-wrap; word-break: break-word;
    color: var(--md-sys-color-on-surface);
    -webkit-overflow-scrolling: touch;
    font-family: ui-monospace, SFMono-Regular, monospace;
}
.license-detail-body::-webkit-scrollbar { width: 4px; }
.license-detail-body::-webkit-scrollbar-track { background: transparent; }
.license-detail-body::-webkit-scrollbar-thumb { background: var(--md-sys-color-outline-variant); border-radius: 4px; }

.toast {
    position: fixed; bottom: clamp(20px,4vw,32px);
    left: 50%; transform: translateX(-50%) translateY(16px);
    background: var(--md-sys-color-inverse-surface);
    color: var(--md-sys-color-inverse-on-surface);
    padding: 14px 24px; border-radius: var(--shape-xs);
    font-size: clamp(.85em,1.4vw,1em); font-weight: 500;
    z-index: 9999; box-shadow: var(--elev-3);
    opacity: 0; transition: opacity var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized), transform var(--md-sys-motion-duration-medium2) var(--md-sys-motion-easing-emphasized);
    pointer-events: none; max-width: 90vw;
}
.toast.show { opacity: 1; transform: translateX(-50%) translateY(0); }

.quiz-screen {
    display: flex; flex-direction: column; align-items: center;
    width: 100%; max-width: 700px;
}

@media (max-height:620px) {
    .quiz-container { padding: 20px 16px 16px; border-radius: var(--shape-lg); }
    #question-word { font-size: 1.5em; margin: 12px 0 20px; min-height: 3em; }
    .option-btn { padding: 12px 14px; gap: 10px; font-size: .9em; }
    .option-btn .opt-label { width: 24px; height: 24px; font-size: .8em; }
    .controls { margin-top: 16px; gap: 8px; }
    .upload-container { padding: 24px 20px; border-radius: var(--shape-lg); }
    .history-card { padding: 16px 20px; border-radius: var(--shape-lg); max-height: 24vh; }
    .action-btns-row { margin-top: 16px; }
}
@media (max-width:380px) {
    body { padding: 12px; }
    .quiz-container { padding: 20px 16px 16px; border-radius: var(--shape-lg); }
    #question-word { font-size: 1.5em; margin: 12px 0 24px; }
    .option-btn { padding: 12px; font-size: .9em; }
    .option-btn .opt-label { width: 24px; height: 24px; font-size: .75em; }
    .ctrl-btn { padding: 10px 16px; font-size: .85em; }
}

/* ── FSRS rating bar ── */
.rating-section {
    margin-top: 16px; padding: 12px 16px;
    background: var(--md-sys-color-surface-container-low);
    border-radius: var(--shape-md); width: 100%;
    box-sizing: border-box;
}
.rating-label {
    font-size: .75em; font-weight: 600; margin-bottom: 8px;
    color: var(--md-sys-color-on-surface-variant); opacity: .7;
    text-transform: uppercase; letter-spacing: .05em;
}
.fsrs-rating-row {
    display: flex; gap: 8px; justify-content: center;
    flex-wrap: wrap;
}
.fsrs-rating-btn {
    display: inline-flex; align-items: center; justify-content: center; gap: 4px;
    padding: 6px 16px; min-height: 36px; border: 2px solid transparent;
    border-radius: var(--shape-full); background: transparent;
    font-size: .82em; font-weight: 500; font-family: inherit;
    color: var(--md-sys-color-on-surface-variant); cursor: pointer;
    transition: all var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard); flex: 1; max-width: 100px;
}
.fsrs-rating-btn.selected {
    border-color: currentColor;
}
.fsrs-btn-again { background: var(--md-sys-color-error-container); color: var(--md-sys-color-on-error-container); }
.fsrs-btn-again.selected { background: var(--md-sys-color-error); color: var(--md-sys-color-on-error); }
.fsrs-btn-hard { background: var(--md-sys-color-tertiary-container); color: var(--md-sys-color-on-tertiary-container); }
.fsrs-btn-hard.selected { background: var(--md-sys-color-tertiary); color: var(--md-sys-color-on-tertiary); }
.fsrs-btn-good { background: var(--md-sys-color-primary-container); color: var(--md-sys-color-on-primary-container); }
.fsrs-btn-good.selected { background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); }
.fsrs-btn-easy { background: var(--md-sys-color-secondary-container); color: var(--md-sys-color-on-secondary-container); }
.fsrs-btn-easy.selected { background: var(--md-sys-color-secondary); color: var(--md-sys-color-on-secondary); }
.fsrs-rating-btn .material-symbols-outlined { font-size: 16px; }
@media (hover: hover) {
    .fsrs-rating-btn:hover:not(.selected) {
        filter: brightness(1.1); box-shadow: var(--elev-1);
    }
}

/* ── Settings tabs ── */
.settings-tabs {
    display: flex; gap: 0; padding: 0 24px; margin-top: 8px;
    border-bottom: 1px solid var(--md-sys-color-outline-variant);
}
.settings-tab {
    flex: 1; padding: 10px 0 8px; border: none; background: transparent;
    font-size: .9em; font-weight: 500; font-family: inherit;
    color: var(--md-sys-color-on-surface-variant); cursor: pointer;
    border-bottom: 2px solid transparent; transition: all var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
    margin-bottom: -1px;
}
.settings-tab.active {
    color: var(--md-sys-color-primary);
    border-bottom-color: var(--md-sys-color-primary);
}

/* ── FSRS settings ── */
.settings-item-sub {
    font-size: .75em; font-weight: 400; margin-top: 2px;
    color: var(--md-sys-color-on-surface-variant); opacity: .7;
}
.fsrs-threshold-section {
    margin-top: 16px; padding: 16px;
    background: var(--md-sys-color-surface-container-low);
    border-radius: var(--shape-md);
}
.fsrs-threshold-header {
    font-size: .85em; font-weight: 600; margin-bottom: 12px;
    color: var(--md-sys-color-on-surface-variant);
}
.fsrs-threshold-grid { display: flex; flex-direction: column; gap: 12px; }
.fsrs-field { display: flex; flex-direction: column; gap: 4px; }
.fsrs-label {
    font-size: .85em; font-weight: 500;
    color: var(--md-sys-color-on-surface);
}
.fsrs-input {
    width: 100%; padding: 10px 12px; border: 2px solid var(--md-sys-color-outline);
    border-radius: var(--shape-xs); background: transparent;
    font-size: .9em; font-family: inherit; outline: none;
    color: var(--md-sys-color-on-surface); transition: border-color var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
    -webkit-user-select: text; user-select: text;
}
.fsrs-input:focus {
    border-color: var(--md-sys-color-primary);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-primary) 20%, transparent);
}
.fsrs-input.error {
    border-color: var(--md-sys-color-error);
    box-shadow: 0 0 0 3px color-mix(in srgb, var(--md-sys-color-error) 20%, transparent);
}
.fsrs-error {
    font-size: .75em; color: var(--md-sys-color-error); font-weight: 500;
}

.finish-screen {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    width: 100%; max-width: 640px; text-align: center;
    animation: fadeUp var(--md-sys-motion-duration-medium4) var(--md-sys-motion-easing-emphasized-decelerate);
    gap: 16px; padding: 48px 24px; flex: 1;
}
.finish-icon {
    font-size: 4em; color: var(--md-sys-color-primary);
    width: 96px; height: 96px; border-radius: 50%;
    background: var(--md-sys-color-primary-container);
    display: flex; align-items: center; justify-content: center;
    margin-bottom: 8px;
}
.finish-title {
    font-size: 1.6em; font-weight: 600; color: var(--md-sys-color-on-surface);
}
.finish-score {
    font-size: 1em; color: var(--md-sys-color-on-surface-variant);
    line-height: 1.6; margin-bottom: 24px;
}
.finish-score .correct { color: var(--md-sys-color-primary); font-weight: 700; }
.finish-score .wrong { color: var(--md-sys-color-error); font-weight: 700; }
.finish-btn {
    width: 100%; max-width: 360px; padding: 14px 24px; min-height: 48px;
    border-radius: var(--shape-full);
    font-size: 1em; font-weight: 500; cursor: pointer; font-family: inherit;
    display: flex; align-items: center; justify-content: center; gap: 8px;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
                box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
                transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.finish-btn:active { transform: scale(0.97); }
.finish-btn.filled { border: none; background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary); }
.finish-btn.filled:hover { box-shadow: var(--elev-1); }
.finish-btn.outlined { border: 1px solid var(--md-sys-color-outline); background: transparent; color: var(--md-sys-color-primary); }
.finish-btn.outlined:hover { background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent); }

/* ── Update dialog ── */
.update-overlay {
    position: fixed; inset: 0; z-index: 2000;
    background: rgba(0,0,0,0.45);
    display: flex; align-items: center; justify-content: center;
    animation: fadeUp var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.update-dialog {
    background: var(--md-sys-color-surface-container);
    border-radius: var(--shape-xxl);
    padding: clamp(24px,5vw,36px);
    text-align: center; max-width: 340px; width: 85%;
    box-shadow: var(--elev-3);
}
.update-title {
    font-size: 1.2em; font-weight: 600; margin-bottom: 12px;
    color: var(--md-sys-color-on-surface);
}
.update-body {
    font-size: .95em; margin-bottom: 24px; line-height: 1.5;
    color: var(--md-sys-color-on-surface-variant);
}
.update-actions {
    display: flex; gap: 12px; justify-content: center;
}
.update-btn {
    flex: 1; padding: 12px 20px; min-height: 44px;
    border-radius: var(--shape-full);
    font-size: .95em; font-weight: 500; cursor: pointer; font-family: inherit;
    transition: background var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
                box-shadow var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard),
                transform var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-standard);
}
.update-btn:active { transform: scale(0.97); }
.update-btn.primary {
    border: none; background: var(--md-sys-color-primary); color: var(--md-sys-color-on-primary);
}
.update-btn.primary:hover { box-shadow: var(--elev-1); }
.update-btn.secondary {
    border: 1px solid var(--md-sys-color-outline); background: transparent; color: var(--md-sys-color-primary);
}
.update-btn.secondary:hover { background: color-mix(in srgb, var(--md-sys-color-primary) 8%, transparent); }

/* ── Download progress inside update dialog ── */
.dl-progress-body {
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 8px;
    margin-bottom: 20px;
    font-size: 1.1em;
    font-weight: 500;
    color: var(--md-sys-color-primary);
}
.update-dl-icon {
    font-size: 28px;
    color: var(--md-sys-color-primary);
}
.dl-track {
    height: 6px;
    background: var(--md-sys-color-surface-container-highest);
    border-radius: 3px;
    overflow: hidden;
    margin-top: 4px;
}
.dl-fill {
    height: 100%;
    background: var(--md-sys-color-primary);
    border-radius: 3px;
    transition: width 250ms cubic-bezier(0.2,0,0,1);
    transform-origin: left;
}

/* ── App shell (NavBar layout) ── */
.app-shell {
    width: 100%; height: 100dvh;
    display: flex; flex-direction: column;
    overflow: hidden;
}
.app-content {
    flex: 1; display: flex; justify-content: center; align-items: center;
    overflow: hidden; position: relative;
    padding: 12px; padding-bottom: calc(12px + var(--safe-bottom));
}

/* ── Bottom navigation bar ── */
.navbar {
    display: flex; justify-content: space-around; align-items: center;
    background: var(--md-sys-color-surface-container);
    border-top: 1px solid var(--md-sys-color-outline-variant);
    padding: 4px 8px;
    padding-bottom: calc(4px + var(--safe-bottom));
    min-height: 64px; z-index: 100; flex-shrink: 0;
}
.nav-item {
    display: flex; flex-direction: column; align-items: center; gap: 2px;
    padding: 6px 16px 4px; border: none; background: transparent;
    font-family: inherit; cursor: pointer; min-width: 64px;
    border-radius: var(--shape-sm);
    transition: background var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
}
@media (hover: hover) {
    .nav-item:hover { background: color-mix(in srgb, var(--md-sys-color-on-surface) 6%, transparent); }
}
.nav-item:active { background: color-mix(in srgb, var(--md-sys-color-on-surface) 12%, transparent); }
.nav-icon {
    font-size: 1.5em; color: var(--md-sys-color-on-surface-variant);
    font-variation-settings: 'FILL' 0;
    transition: color var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
}
.nav-item.active .nav-icon {
    color: var(--md-sys-color-primary);
    font-variation-settings: 'FILL' 1;
}
.nav-label {
    font-size: .65em; font-weight: 500;
    color: var(--md-sys-color-on-surface-variant);
    transition: color var(--md-sys-motion-duration-short3) var(--md-sys-motion-easing-standard);
}
.nav-item.active .nav-label {
    color: var(--md-sys-color-primary);
}

/* ── Library screen ── */
.library-screen {
    display: flex; flex-direction: column; align-items: center; justify-content: center;
    flex: 1; text-align: center; gap: 16px; padding: 24px;
    animation: fadeUp var(--md-sys-motion-duration-long3) var(--md-sys-motion-easing-emphasized-decelerate);
}
.library-icon {
    font-size: 3.5em; color: var(--md-sys-color-on-surface-variant); opacity: .5;
    width: 96px; height: 96px; border-radius: 50%;
    background: var(--md-sys-color-surface-container-high);
    display: flex; align-items: center; justify-content: center;
}
.library-title {
    font-size: 1.3em; font-weight: 600; color: var(--md-sys-color-on-surface);
}
.library-subtitle {
    font-size: .9em; color: var(--md-sys-color-on-surface-variant); line-height: 1.5;
    max-width: 280px;
}

/* ── Settings screen ── */
.settings-screen {
    align-self: stretch; width: 100%;
    display: flex; flex-direction: column;
    overflow: hidden;
    animation: fadeUp var(--md-sys-motion-duration-short4) var(--md-sys-motion-easing-emphasized-decelerate);
}
"#;
