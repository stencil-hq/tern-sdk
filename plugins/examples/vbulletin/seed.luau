--!strict
-- Seed content for the vBulletin board recreation: users, forum tree and threads.
-- Times are relative to the board's epoch ("now"): `joined` in days, `last`/`ago` in minutes.

return {
	title = "Tern Community Forums",

	users = {
		{ id = 1, name = "Kestrel", group = "admin", usertitle = "Head Honcho", joined = 3310, posts = 8412, location = "Somewhere with fiber", occupation = "Herding pixels", interests = "GPUs, typography, birds", bio = "Started this board back when Tern was a weekend project. Still running it on the same box.", sig = "[b]Tern Community Forums[/b] — read the [url=https://docs.stencil.so/tern/]docs[/url] before you post :)", avatar = "tern", online = true, last = 2, birthday = "04-17" },
		{ id = 2, name = "bytewarden", group = "smod", usertitle = "Keeper of the Bits", joined = 3120, posts = 5230, location = "Rotterdam", occupation = "SRE", interests = "Observability, cycling", sig = "[i]\"It works on my machine\" is not a status update.[/i]", avatar = "robot", online = false, last = 95, birthday = "11-02" },
		{ id = 3, name = "PaneInTheGlass", group = "mod", joined = 2810, posts = 3904, location = "Leeds, UK", occupation = "Backend dev", interests = "Split panes, tea, bad puns", sig = "Moderator of Help & Support. PM me if a thread needs moving. [color=Gray]No, I won't fix your dotfiles.[/color]", avatar = "penguin", online = true, last = 0 },
		{ id = 4, name = "luau_larry", group = "mod", usertitle = "Plugin Wrangler", joined = 2505, posts = 3120, location = "Austin, TX", occupation = "Game dev", interests = "Luau, type checkers, BBQ", sig = "[code]--!strict or bust[/code]", avatar = "cat", online = false, last = 210, birthday = "07-30" },
		{ id = 5, name = "xX_GPU_Slayer_Xx", group = "banned", usertitle = "Banned", joined = 214, posts = 45, location = "ur mom's basement", interests = "benchmarks", online = false, last = 52000 },
		{ id = 6, name = "crtmonk", group = "member", joined = 3205, posts = 3488, location = "Ohio", occupation = "Sysadmin", interests = "Phosphor, scanlines, bourbon", bio = "Been on terminals since the green screen days.", sig = "[color=Green]> READY.[/color] | 80x24 forever", avatar = "crt", online = true, last = 6 },
		{ id = 7, name = "foxtrot_dev", group = "member", joined = 1502, posts = 902, location = "Berlin", occupation = "Frontend dev", interests = "CSS, synths", sig = "Theme: [url=https://example.com/foxglow]Foxglow[/url] | \"CSS is a programming language, fight me\"", avatar = "rocket", online = false, last = 340 },
		{ id = 8, name = "nullpointer88", group = "member", joined = 2203, posts = 1714, location = "Warsaw", occupation = "Systems programmer", interests = "Rust, Zig, segfaults", sig = "Segmentation fault (core dumped) :rolleyes:", avatar = "skull", online = false, last = 1600 },
		{ id = 9, name = "quietkeys", group = "member", joined = 905, posts = 402, location = "Kyoto", occupation = "Writer", interests = "Keyboards, ink", avatar = "keyboard", online = true, last = 11 },
		{ id = 10, name = "mangoshell", group = "member", joined = 702, posts = 263, location = "Manila", occupation = "Student", interests = "fish shell, mangoes", sig = "[b]fish[/b] > everything :p", avatar = "coffee", online = false, last = 700 },
		{ id = 11, name = "zsh_witch", group = "member", joined = 1108, posts = 611, location = "Portland", occupation = "DevOps", interests = "Shell scripting, tarot", sig = "[i]setopt extended_glob[/i] and the world opens up", avatar = "ghost", online = false, last = 2300, birthday = "10-31" },
		{ id = 12, name = "floppydisco", group = "member", joined = 3010, posts = 2104, location = "Montréal", occupation = "Archivist", interests = "Floppies, disco, BBSes", sig = "1.44MB ought to be enough for anybody", avatar = "floppy", online = false, last = 4100 },
		{ id = 13, name = "ninjadiff", group = "member", joined = 604, posts = 181, location = "Seoul", occupation = "Code reviewer", interests = "Reel, git internals", sig = "LGTM :thumbsup: (I didn't read it)", avatar = "ninja", online = true, last = 3 },
		{ id = 14, name = "newb_tim", group = "member", joined = 3, posts = 0, location = "Ireland", online = false, last = 400 },
		{ id = 15, name = "orbital_kat", group = "member", joined = 12, posts = 2, location = "Porto, Portugal", occupation = "Data analyst", interests = "Space, Python", online = true, last = 14 },
		{ id = 16, name = "tilde_tom", group = "member", joined = 403, posts = 88, location = "Melbourne", interests = "Home labs", sig = "~ is where the heart is", online = false, last = 3000 },
		{ id = 17, name = "deadbeef", group = "member", joined = 1810, posts = 749, location = "Helsinki", occupation = "Firmware", interests = "Hex, saunas", online = false, last = 150 },
		{ id = 18, name = "gridlocked", group = "member", joined = 41, posts = 11, location = "Chicago", online = false, last = 900 },
		{ id = 19, name = "vt220fan", group = "member", usertitle = "Escape Sequence Enthusiast", joined = 2620, posts = 1302, location = "Vermont", occupation = "Retired EE", interests = "DEC hardware, ham radio", online = false, last = 260 },
		{ id = 20, name = "pixel_pusher", group = "member", joined = 302, posts = 54, location = "Lyon", occupation = "Designer", online = false, last = 5200 },
	},

	forums = {
		{ id = 1, title = "Tern Community", kind = "category" },
		{ id = 2, parent = 1, title = "Announcements", desc = "Release news and board announcements from the team.", kind = "forum" },
		{ id = 3, parent = 1, title = "General Discussion", desc = "Anything and everything Tern. Panes, tabs, workflows, opinions.", kind = "forum" },
		{ id = 4, parent = 1, title = "Introductions", desc = "New here? Say hello!", kind = "forum" },
		{ id = 5, title = "Help & Support", kind = "category" },
		{ id = 6, parent = 5, title = "Installation & Setup", desc = "Getting Tern installed and configured.", kind = "forum" },
		{ id = 7, parent = 6, title = "macOS", desc = "Homebrew, dmg, Gatekeeper and friends.", kind = "forum" },
		{ id = 8, parent = 6, title = "Linux", desc = "Distros, packages, Wayland and X11.", kind = "forum" },
		{ id = 9, parent = 6, title = "Windows", desc = "Native builds and WSL.", kind = "forum" },
		{ id = 10, parent = 5, title = "Troubleshooting", desc = "Something not working? Post your logs here.", kind = "forum" },
		{ id = 11, parent = 5, title = "Bug Reports", desc = "Found a bug in Tern, ternd, Reel or the iOS app? Report it here.", kind = "forum" },
		{ id = 12, title = "Extending Tern", kind = "category" },
		{ id = 13, parent = 12, title = "Plugins & Blocks", desc = "Writing Luau plugins and blocks.", kind = "forum" },
		{ id = 14, parent = 13, title = "Command Lenses", desc = "Lenses that reshape command output.", kind = "forum" },
		{ id = 15, parent = 13, title = "Themes & CSS", desc = "Share themes, ask about styling.", kind = "forum" },
		{ id = 16, parent = 12, title = "Feature Requests", desc = "Wishlist. Search first, +1 second.", kind = "forum" },
		{ id = 17, parent = 12, title = "Tern Documentation", desc = "The official docs (external link).", kind = "link", link = "https://docs.stencil.so/tern/" },
		{ id = 18, title = "Off Topic", kind = "category" },
		{ id = 19, parent = 18, title = "The Lounge", desc = "Kick back. Non-Tern chatter.", kind = "forum" },
		{ id = 20, parent = 18, title = "Battlestations & Hardware", desc = "Desks, monitors, keyboards, GPUs.", kind = "forum" },
		{ id = 21, parent = 18, title = "Retro Computing", desc = "Old iron, real terminals, BBS nostalgia.", kind = "forum" },
		{ id = 22, parent = 18, title = "Archive", desc = "Old threads from the beta days. Read only.", kind = "forum", closed = true },
	},

	threads = {
		-- Announcements
		{ id = 1, forum = 2, title = "Forum Rules & Posting Guidelines - READ BEFORE POSTING", icon = "exclamation", sticky = true, locked = true, views = 24810, posts = {
			{ user = 1, ago = 1500000, icon = "exclamation", body = [=[
Welcome to the [b]Tern Community Forums[/b]! A few ground rules so this place stays nice:

[list=1]
[*][b]Be civil.[/b] Disagree with ideas, not people.
[*][b]Search before posting.[/b] Odds are someone hit your problem already.
[*][b]Post in the right forum.[/b] Help goes in [i]Help & Support[/i], plugin talk in [i]Extending Tern[/i].
[*][b]Include versions and logs[/b] when asking for help (Tern version, OS, GPU).
[*]No warez, no spam, no "first!!!" posts.
[/list]

[color=Red]Repeat offenders will be banned without warning.[/color]

Have fun :)]=] },
		} },
		{ id = 2, forum = 2, title = "Tern 2.0 released: Reel, the iOS app and plugin API v3", icon = "lightbulb", sticky = true, views = 18240, rating = 4.8, votes = 41, posts = {
			{ user = 1, ago = 262000, icon = "lightbulb", body = [=[
[size=4][color=DarkBlue][b]Tern 2.0 is out![/b][/color][/size]

Highlights:
[list]
[*][b]Reel[/b] - a side-by-side diff review tool, built in. Run [code]tern reel main..HEAD[/code]
[*][b]iOS app[/b] - attach to your sessions from your phone via the session daemon
[*][b]Plugin API v3[/b] - blocks, command lenses and theme CSS, all in Luau
[*]Renderer rewrite: glyph atlas is now shared across panes, ~40% less VRAM
[/list]

Full changelog in the [url=https://docs.stencil.so/tern/]docs[/url]. Report bugs in [i]Bug Reports[/i], not here please ;)]=] },
			{ user = 6, ago = 261900, body = "Finally! Been waiting for Reel since the beta. Upgrading right now :D" },
			{ user = 7, ago = 261000, body = "Theme CSS in v3 is [b]so[/b] much nicer. Already porting Foxglow." },
			{ user = 8, ago = 250000, body = [=[
[quote=Kestrel]~40% less VRAM[/quote]
Can confirm, 12 panes on my laptop no longer make the fans spin up. Great work.]=] },
			{ user = 1, ago = 249000, body = "Thanks all :) Keep the reports coming." },
		} },
		{ id = 3, forum = 2, title = "Scheduled downtime: forum database migration Sunday", icon = "page", views = 912, posts = {
			{ user = 1, ago = 2700, body = "Heads up: the board will be read-only for about an hour on Sunday while we move to the new database server. Your posts are safe. [i]Probably.[/i] :p" },
			{ user = 2, ago = 2650, body = "Backups verified twice. The \"probably\" is a joke, folks." },
			{ user = 9, ago = 2000, body = "Thanks for the warning! Will the search index get rebuilt too? It's been flaky." },
		} },

		-- General Discussion
		{ id = 4, forum = 3, title = "General Discussion FAQ: search before you post", icon = "exclamation", sticky = true, views = 9120, posts = {
			{ user = 2, ago = 1200000, body = [=[
Common questions we see every week:

[b]Q: Where is my config?[/b] [code]~/.config/tern/config.luau[/code]
[b]Q: Does Tern support Windows?[/b] See the Windows sub-forum.
[b]Q: Why GPU?[/b] Smooth scrolling and huge scrollback without stutter. Search "GPU worth it" for the 40-page debate.

Duplicate threads will be merged.]=] },
		} },
		{ id = 5, forum = 3, title = "Is GPU rendering in a terminal actually worth it?", icon = "question", views = 3410, rating = 3.5, votes = 8, posts = {
			{ user = 16, ago = 900, body = "Honest question. My old terminal was fine. What do I actually gain besides a bigger number in a benchmark?" },
			{ user = 6, ago = 640, body = "Try [code]cat[/code]-ing a 200MB log in both. Then come back :cool:" },
			{ user = 8, ago = 300, body = [=[
[quote=tilde_tom]What do I actually gain besides a bigger number in a benchmark?[/quote]
Latency, mostly. Keystroke-to-photon is noticeably lower, and resizing with 10 panes doesn't tear.]=] },
			{ user = 16, ago = 95, body = "OK the cat test convinced me :eek:" },
		} },
		{ id = 6, forum = 3, title = "Tabs vs panes - how do you organise?", views = 1220, posts = {
			{ user = 9, ago = 15000, body = "I do one tab per project, panes inside for editor / server / logs. Curious what others do." },
			{ user = 11, ago = 14200, body = "One tab per [i]host[/i] for me. Panes are for tail -f and htop." },
			{ user = 3, ago = 13800, body = "Panes all the way down. Tabs are just panes that gave up :p" },
		} },
		{ id = 7, forum = 3, title = "What does the session daemon actually do?", icon = "question", views = 2280, posts = {
			{ user = 20, ago = 2600, body = "I see [code]ternd[/code] in my process list. Is it needed? What happens if I kill it?" },
			{ user = 3, ago = 2500, body = "It owns the PTYs. Your shells keep running when you close the window, and the iOS app attaches to it. Kill it and your sessions die with it." },
			{ user = 17, ago = 2100, body = "Think tmux server, but the protocol is binary and the renderer is the client." },
			{ user = 20, ago = 1700, body = "Ah, that makes sense. Thanks both!" },
		} },
		{ id = 8, forum = 3, title = "Reel replaced my git diff workflow", icon = "thumbsup", views = 1980, rating = 4.5, votes = 6, posts = {
			{ user = 13, ago = 22000, body = "Been using [b]Reel[/b] for code review for a month. Inline comments, word-level highlighting, jump between hunks with n/p. I don't open the web UI anymore." },
			{ user = 7, ago = 21500, body = "Does it handle renames? My last tool showed them as delete+add." },
			{ user = 13, ago = 21000, body = "[quote=foxtrot_dev]Does it handle renames?[/quote]\nYes, with a similarity %. Feels very git-native." },
		} },
		{ id = 9, forum = 3, title = "Anyone else using Tern on iPad?", views = 860, posts = {
			{ user = 9, ago = 45000, body = "With a keyboard case the iOS app is surprisingly usable. Attach to my desktop sessions from the couch." },
			{ user = 10, ago = 44000, body = "Only on phone, and only for checking builds. Tiny but works." },
			{ user = 6, ago = 43000, body = "iPad + ssh + ternd = I never get up anymore. My doctor is concerned." },
		} },
		{ id = 10, forum = 3, title = "Font recommendations? Ligatures yes/no", icon = "talking", views = 2950, posts = {
			{ user = 18, ago = 60000, body = "Looking for a good mono font. Do people actually like ligatures in a terminal?" },
			{ user = 7, ago = 59000, body = "Ligatures in the editor yes, terminal no. [code]!=[/code] turning into ≠ in a shell is cursed." },
			{ user = 19, ago = 58000, body = "Any bitmap-looking font at 13px. Real men use 9x15 :cool:" },
			{ user = 9, ago = 57000, body = "[quote=vt220fan]Real men use 9x15[/quote]\nAnd real eyes need glasses by 40 :p" },
		} },
		{ id = 11, forum = 3, title = "ternd using 2% CPU at idle - normal?", views = 640, posts = {
			{ user = 14, ago = 1300, body = "hi, is it normal that ternd uses 2% cpu when im not doing anything? sorry if wrong place" },
			{ user = 17, ago = 1250, body = "Do you have a pane running htop or a clock in your prompt? It's redrawing those." },
			{ user = 14, ago = 1200, body = "oh. yes. the clock. thanks :redface:" },
		} },
		{ id = 12, forum = 3, title = "Tern vs everything else - honest comparison", icon = "angry", locked = true, views = 6120, posts = {
			{ user = 5, ago = 70000, body = "tern is BLOATED garbage. my terminal does 10000 fps. yours do 60. cope :mad: :mad:" },
			{ user = 8, ago = 69900, body = "Your monitor does 60, champ." },
			{ user = 6, ago = 69800, body = ":D :D :D" },
			{ user = 5, ago = 69700, body = "lol ok boomer. enjoy ur electron app" },
			{ user = 3, ago = 69500, icon = "exclamation", body = "It isn't Electron, and this isn't a comparison thread anymore. [b]Locked.[/b] xX_GPU_Slayer_Xx, that's your third warning." },
		} },
		{ id = 13, forum = 3, title = "Bump: does anyone have the old 1.x config docs?", views = 1030, posts = {
			{ user = 16, ago = 110000, body = "Need to migrate a coworker's ancient config. The 1.x docs are gone from the site." },
			{ user = 16, ago = 100000, body = "bump" },
			{ user = 12, ago = 99000, body = "I mirrored them years ago. Check the Archive forum, the beta thread links a tarball. Or PM me." },
		} },
		{ id = 14, forum = 3, title = "What's your favourite keybinding?", icon = "smile", views = 4870, rating = 4.2, votes = 11, posts = {
			{ user = 9, ago = 4300, body = "Mine: [code]keys.bind(\"cmd+d\", \"pane.split_right\")[/code] Simple, but I hit it a hundred times a day." },
			{ user = 6, ago = 4100, body = "cmd+k to clear. Muscle memory from 2006." },
			{ user = 11, ago = 3900, body = "ctrl+shift+arrow to resize panes. Life changing." },
			{ user = 4, ago = 3500, body = [=[
I bound a Luau function to jump to the last pane with an error exit code:
[code]keys.bind("cmd+e", function(ctx)
	local p = ctx.panes:find(function(p) return p.last_exit ~= 0 end)
	if p then p:focus() end
end)[/code]]=] },
			{ user = 13, ago = 3000, body = "[quote=luau_larry]jump to the last pane with an error exit code[/quote]\nStealing this immediately :thumbsup:" },
			{ user = 10, ago = 2400, body = "alt+1..9 for tabs, like in browsers" },
			{ user = 17, ago = 1900, body = "+1 for luau_larry's error jumper. Works great." },
			{ user = 15, ago = 1100, body = "Newbie here, where do these go? The config file?" },
			{ user = 3, ago = 1000, body = "Yep, [code]~/.config/tern/config.luau[/code]. It reloads on save." },
			{ user = 15, ago = 900, body = "Wow it really does reload instantly. :D" },
			{ user = 19, ago = 400, body = "ctrl+s bound to nothing so I stop freezing my terminal with XOFF. 40 years of habit." },
			{ user = 6, ago = 60, body = "[quote=vt220fan]so I stop freezing my terminal with XOFF[/quote]\nThe true veteran's binding :D" },
			{ user = 9, ago = 3, body = "This thread got me to clean up my whole config. Thanks everyone!" },
		} },
		{ id = 15, forum = 3, title = "Scrollback size - how big do you go?", views = 720, posts = {
			{ user = 17, ago = 130000, body = "I set 1,000,000 lines. Tern doesn't care. Anyone gone higher?" },
			{ user = 8, ago = 129000, body = "Unlimited. It spills to disk past 200MB, didn't notice any slowdown." },
			{ user = 20, ago = 128000, body = "10k like a normal person :p" },
		} },
		{ id = 16, forum = 3, title = "Plugin marketplace when?", icon = "question", views = 1490, posts = {
			{ user = 10, ago = 160000, body = "Would love a way to browse and install plugins from inside Tern." },
			{ user = 4, ago = 159000, body = "It's on the roadmap. For now, git clone into the plugins folder works fine." },
		} },
		{ id = 17, forum = 3, title = "Tern turns 3! Happy birthday", icon = "smile", views = 2210, rating = 5, votes = 9, posts = {
			{ user = 6, ago = 525000, body = "Three years since the first public build. Who remembers the 0.3 renderer that drew everything in magenta? :D" },
			{ user = 12, ago = 524500, body = "I have a screenshot somewhere. On a floppy, naturally." },
			{ user = 1, ago = 524000, body = "Thanks for sticking around everyone. This board is half the reason we kept going :)" },
			{ user = 19, ago = 523000, body = "Happy birthday Tern!" },
		} },
		{ id = 18, forum = 3, title = "Quake-style dropdown mode", views = 560, posts = {
			{ user = 16, ago = 200000, body = "Is there a hotkey window mode like the old quake consoles?" },
			{ user = 3, ago = 199000, body = "Yes, set [code]window.dropdown = true[/code] and pick a global hotkey. Moving this to Help... actually it's answered, leaving it." },
		} },
		{ id = 19, forum = 3, title = "How do you name your sessions?", views = 470, posts = {
			{ user = 11, ago = 75000, body = "Mine are all tarot cards. 'the-tower' is prod, obviously." },
			{ user = 17, ago = 74000, body = "Hex. 0xbeef, 0xcafe, 0xf00d." },
			{ user = 10, ago = 73000, body = "fruits :p mango, papaya, durian (the scary one)" },
		} },
		{ id = 20, forum = 3, title = "The command palette is underrated", icon = "lightbulb", views = 680, posts = {
			{ user = 7, ago = 85000, body = "ctrl+shift+P and fuzzy-type anything. Every plugin action shows up there too. Took me a year to notice." },
			{ user = 9, ago = 84000, body = "Same! Discovered half the features through it." },
		} },
		{ id = 21, forum = 3, title = "Copy mode with vim keys?", icon = "question", views = 990, posts = {
			{ user = 18, ago = 33000, body = "Coming from tmux. Is there a vi copy mode?" },
			{ user = 11, ago = 32500, body = "ctrl+shift+space enters copy mode, hjkl/w/b work, v to select, y to yank." },
			{ user = 18, ago = 32000, body = "Perfect, thanks!" },
		} },
		{ id = 22, forum = 3, title = "Light or dark theme?", views = 1340, posts = {
			{ user = 20, ago = 230000, body = "Light theme gang, where you at?" },
			{ user = 6, ago = 229000, body = "Green on black. This is not up for debate." },
			{ user = 7, ago = 228000, body = "Auto-switch with the OS. Best of both." },
			{ user = 20, ago = 227000, body = "[quote=crtmonk]This is not up for debate.[/quote]\nThis is literally a debate thread :p" },
		} },
		{ id = 23, forum = 3, title = "Startup time numbers on my machine", icon = "page", views = 820, posts = {
			{ user = 8, ago = 300000, body = "[code]cold: 142ms\nwarm:  38ms\nwith 6 plugins: 51ms[/code] M2, release build." },
			{ user = 17, ago = 299000, body = "Similar here on Linux, ~45ms warm." },
			{ user = 4, ago = 298000, body = "Plugins load lazily on first use since v3, so the plugin count barely matters." },
		} },
		{ id = 24, forum = 3, title = "Using Tern to ssh into 40 servers", views = 1150, posts = {
			{ user = 2, ago = 400000, body = "I have a layout that opens 40 panes, one per box, with input broadcast. Terrifying, but fast." },
			{ user = 11, ago = 399000, body = "Please share that layout file. Please." },
			{ user = 2, ago = 398000, body = "It's just a loop over a host list calling [code]pane.split[/code]. I'll post it in Plugins." },
		} },
		{ id = 25, forum = 3, title = "Do Reel review comments sync to the iOS app?", icon = "question", views = 330, posts = {
			{ user = 13, ago = 36000, body = "Left comments in Reel on the desktop, don't see them on the phone." },
			{ user = 3, ago = 35000, body = "Not yet. Comments are stored in the repo's .git/reel folder, the app only reads session state." },
		} },
		{ id = 26, forum = 3, title = "Lost my layout after update :(", icon = "unhappy", views = 410, posts = {
			{ user = 10, ago = 2300, body = "Updated and all my saved layouts are gone :( :(" },
			{ user = 3, ago = 2200, body = "They moved to [code]~/.local/state/tern/layouts[/code]. Check there, the old ones get migrated on first launch." },
			{ user = 10, ago = 2000, body = "Found them! Panic over :D" },
		} },
		{ id = 27, forum = 3, title = "What shell are you running in Tern?", views = 1790, posts = {
			{ user = 10, ago = 600, body = "fish, obviously :p" },
			{ user = 11, ago = 450, body = "zsh with way too many plugins." },
			{ user = 19, ago = 200, body = "bash 3.2 because that's what came with the machine." },
			{ user = 13, ago = 40, body = "nushell. The command lenses work really well with its tables." },
		} },
		{ id = 28, forum = 3, title = "Thank you devs", icon = "thumbsup", views = 380, posts = {
			{ user = 18, ago = 18000, body = "Just wanted to say: switched from three different tools to Tern + Reel and my day is calmer. Thanks." },
			{ user = 1, ago = 17500, body = "That's lovely to hear, thank you :)" },
		} },

		-- Introductions
		{ id = 29, forum = 4, title = "Hi from Portugal!", icon = "smile", views = 120, posts = {
			{ user = 15, ago = 17000, body = "Hello! Data analyst from Porto, just installed Tern yesterday. Mostly use Python and SQL. Nice to meet you all :)" },
			{ user = 6, ago = 16500, body = "Welcome! Tip: grab the Python block from the Plugins forum." },
			{ user = 3, ago = 180, body = "Welcome aboard orbital_kat! Shout if you get stuck." },
		} },
		{ id = 30, forum = 4, title = "newb_tim says hi", views = 48, posts = {
			{ user = 14, ago = 2800, body = "hi everyone. new to terminals in general to be honest. my friend said to try this" },
			{ user = 9, ago = 2500, body = "Welcome! No question is too basic in Help & Support, go for it." },
		} },
		{ id = 31, forum = 4, title = "Long time lurker, first post", views = 210, posts = {
			{ user = 18, ago = 58000, body = "Been reading for a year. Finally registered. I build CI pipelines for a living." },
			{ user = 2, ago = 57500, body = "Welcome out of the shadows :cool:" },
			{ user = 8, ago = 57000, body = "A CI person! We need you in Bug Reports." },
		} },

		-- Installation & Setup
		{ id = 32, forum = 6, title = "Building from source - which toolchain?", icon = "question", views = 1430, posts = {
			{ user = 17, ago = 140000, body = "Trying to build on a weird distro. What compiler version does the build expect?" },
			{ user = 8, ago = 139000, body = "[code]rustup toolchain install stable\ncargo build --release -p tern[/code] The pinned version is in rust-toolchain.toml." },
			{ user = 17, ago = 138000, body = "Worked first try. Thanks!" },
		} },
		{ id = 33, forum = 6, title = "Where is the config file?", icon = "question", views = 2620, posts = {
			{ user = 14, ago = 4000, body = "sorry for stupid question but where do i put settings" },
			{ user = 3, ago = 3800, body = "Not stupid! [code]~/.config/tern/config.luau[/code] - create it if it doesn't exist. Also see the FAQ sticky in General Discussion." },
		} },
		{ id = 34, forum = 7, title = "Homebrew cask vs dmg", views = 940, posts = {
			{ user = 20, ago = 95000, body = "Any difference between installing via brew and the dmg?" },
			{ user = 9, ago = 94000, body = "Same binary. brew is easier to upgrade: [code]brew upgrade --cask tern[/code]" },
		} },
		{ id = 35, forum = 7, title = "Gatekeeper warning on first launch", icon = "exclamation", views = 1210, posts = {
			{ user = 15, ago = 16000, body = "macOS says it can't verify the developer?" },
			{ user = 3, ago = 15500, body = "That was a nightly build, which isn't notarized. Grab the stable release and it opens fine." },
			{ user = 15, ago = 15000, body = "Oops, I clicked the wrong download. Works now, thanks!" },
		} },
		{ id = 36, forum = 8, title = "Wayland: blurry text with fractional scaling", views = 2040, rating = 4, votes = 3, posts = {
			{ user = 16, ago = 250000, body = "On GNOME at 125% the text is soft. X11 is fine." },
			{ user = 8, ago = 249000, body = "Make sure your compositor supports wp_fractional_scale_v1. Older GNOME doesn't, it upscales." },
			{ user = 16, ago = 248000, body = "Updated GNOME, crisp now. You're a legend." },
		} },
		{ id = 37, forum = 8, title = "AUR package outdated?", views = 720, posts = {
			{ user = 11, ago = 50000, body = "The AUR tern package is two minors behind." },
			{ user = 17, ago = 49000, body = "Use tern-bin, it tracks releases automatically." },
		} },
		{ id = 38, forum = 9, title = "Windows support status?", icon = "question", views = 3310, posts = {
			{ user = 20, ago = 320000, body = "Is a native Windows build planned?" },
			{ user = 1, ago = 319000, body = "Yes. ConPTY backend is working internally, renderer runs on DX12. No date yet." },
			{ user = 5, ago = 318000, body = "windows users dont deserve good terminals lol" },
			{ user = 3, ago = 317900, body = "Off-topic and rude. Infraction issued." },
		} },
		{ id = 39, forum = 9, title = "WSL2 integration tips", icon = "page", views = 1580, posts = {
			{ user = 11, ago = 120000, body = [=[
Running Tern on Linux under WSLg works. Tips:
[list]
[*]Use the GPU drivers from the host, not mesa llvmpipe
[*]Set [code]renderer.vsync = false[/code] if input feels laggy
[/list]]=] },
			{ user = 16, ago = 119000, body = "The vsync tip fixed it for me. Bookmarked." },
		} },

		-- Troubleshooting
		{ id = 40, forum = 10, title = "Before posting: how to collect a ternd log", icon = "exclamation", sticky = true, views = 7400, posts = {
			{ user = 3, ago = 700000, body = [=[
Please attach logs with your help request:
[code]TERN_LOG=debug tern 2> ~/tern.log
ternd --log-file ~/ternd.log --verbose[/code]
Include your Tern version ([code]tern --version[/code]), OS, and GPU. Threads without this will get a link to this post :p]=] },
		} },
		{ id = 41, forum = 10, title = "Panes go black after waking from sleep", icon = "unhappy", views = 860, posts = {
			{ user = 18, ago = 5000, body = "After sleep, some panes render black until I resize the window. Linux, AMD." },
			{ user = 3, ago = 4800, body = "Logs please (see the sticky). Sounds like a lost GPU device." },
			{ user = 18, ago = 4600, body = "[code]WARN renderer: surface lost, recreating\nERROR atlas: texture upload failed[/code]" },
			{ user = 8, ago = 1500, body = "Same here. Workaround: [code]renderer.recover_on_resume = true[/code] in config." },
			{ user = 18, ago = 25, body = "That works. Leaving this open in case someone finds a proper fix." },
		} },
		{ id = 42, forum = 10, title = "Box-drawing characters misaligned", views = 540, posts = {
			{ user = 20, ago = 78000, body = "My htop borders have gaps between lines." },
			{ user = 7, ago = 77000, body = "Turn on [code]font.builtin_box_drawing = true[/code] - Tern draws them itself instead of using the font." },
			{ user = 20, ago = 76000, body = "Perfect. :)" },
		} },

		-- Bug Reports
		{ id = 43, forum = 11, title = "Reel crashes on binary files in diff", icon = "exclamation", views = 690, posts = {
			{ user = 13, ago = 3200, body = "Opening a range that includes a .png crashes Reel. Repro: commit any image, [code]tern reel HEAD~1..HEAD[/code]" },
			{ user = 2, ago = 3100, body = "Reproduced on 2.3.1. Filed internally, thanks for the clean repro." },
			{ user = 13, ago = 280, body = "Fixed in the nightly, binary files now show as \"Binary file changed\". :thumbsup:" },
		} },
		{ id = 44, forum = 11, title = "iOS app disconnects after 5 min in background", views = 540, posts = {
			{ user = 9, ago = 2000, body = "Switch to another app for ~5 min and the session shows disconnected. Have to reattach." },
			{ user = 2, ago = 1900, body = "That's iOS suspending the socket. Reattach should be automatic though - is it not?" },
			{ user = 9, ago = 1800, body = "It reattaches, but the scrollback is empty until I scroll. Minor, but odd." },
		} },
		{ id = 45, forum = 11, title = "Tab title not updating with OSC 2", views = 300, posts = {
			{ user = 19, ago = 190000, body = "[code]printf '\\e]2;hello\\a'[/code] doesn't change the tab title." },
			{ user = 4, ago = 189000, body = "Check if a plugin overrides the title. Disable plugins and retry?" },
			{ user = 19, ago = 188000, body = "Yep, my git-branch plugin was clobbering it. Not a bug, my bad :redface:" },
		} },

		-- Plugins & Blocks
		{ id = 46, forum = 13, title = "Plugin API v3 quickstart (blocks)", icon = "lightbulb", sticky = true, views = 11200, rating = 4.9, votes = 23, posts = {
			{ user = 4, ago = 255000, body = [=[
A minimal block:
[code]--!strict
local tern = require("@tern")

return tern.block({
	title = "Hello",
	render = function(ctx)
		return tern.text("Hello from Luau!")
	end,
})[/code]
Drop it in [b]~/.config/tern/plugins/hello/init.luau[/b] and it shows up in the block picker. Questions below.]=] },
			{ user = 7, ago = 254000, body = "Can blocks receive keyboard focus?" },
			{ user = 4, ago = 253500, body = "[quote=foxtrot_dev]Can blocks receive keyboard focus?[/quote]\nYes, return [code]focusable = true[/code] and handle ctx.key events." },
			{ user = 10, ago = 200000, body = "Thank you for this, made my first block in 10 minutes :D" },
		} },
		{ id = 47, forum = 13, title = "My first block: a clock widget", icon = "smile", views = 410, rating = 4, votes = 2, posts = {
			{ user = 15, ago = 1400, body = [=[
Made a tiny clock block!
[code]render = function(ctx)
	ctx:every(1)
	return tern.text(os.date("%H:%M:%S"))
end[/code]
Is redrawing every second bad for performance?]=] },
			{ user = 4, ago = 1300, body = "Nope, a single text node per second is free. Nice work! :thumbsup:" },
			{ user = 6, ago = 60, body = "Now do one with a blinking colon :cool:" },
		} },
		{ id = 48, forum = 13, title = "How to persist state across reloads?", icon = "question", views = 760, posts = {
			{ user = 7, ago = 40000, body = "My block loses its state every time I save the plugin file." },
			{ user = 4, ago = 39500, body = "Use [code]ctx.store:get(key)[/code] / [code]ctx.store:set(key, value)[/code]. It's persisted per block instance." },
			{ user = 2, ago = 39000, body = "And the 40-servers layout script I promised: [code]for i, host in hosts do\n\tctx.pane:split(\"right\"):run(\"ssh \" .. host)\nend[/code]" },
		} },
		{ id = 49, forum = 14, title = "Lens for kubectl output", icon = "lightbulb", views = 980, rating = 4.3, votes = 4, posts = {
			{ user = 11, ago = 2600, body = "Wrote a lens that turns [code]kubectl get pods[/code] into a sortable table with status colours. Repo link in my profile." },
			{ user = 17, ago = 2400, body = "Does it handle -o wide?" },
			{ user = 11, ago = 1600, body = "It does now :p" },
		} },
		{ id = 50, forum = 14, title = "Lens not triggering on piped commands", icon = "question", views = 360, posts = {
			{ user = 10, ago = 66000, body = "My lens matches 'ls' but not 'ls | head'." },
			{ user = 4, ago = 65000, body = "Lens matching uses the first command in the pipeline unless you set [code]match = \"last\"[/code]. Output comes from the last one, so that's what you want." },
		} },
		{ id = 51, forum = 15, title = "[Theme] Solarized-ish with proper dim colours", icon = "cool", views = 1700, rating = 4.6, votes = 7, posts = {
			{ user = 7, ago = 110500, body = [=[
Here's my take, with dim colours that are actually readable:
[code].tern-pane {
	--fg: #93a1a1;
	--bg: #002b36;
	--dim: #586e75;
}
.tern-tab[data-active] { border-bottom: 2px solid #b58900; }[/code]]=] },
			{ user = 9, ago = 110000, body = "Gorgeous. Using it now." },
			{ user = 20, ago = 109000, body = "Any light variant? :p" },
		} },
		{ id = 52, forum = 15, title = "CSS: can I change the tab bar height?", icon = "question", views = 520, posts = {
			{ user = 20, ago = 26000, body = "Tab bar is too tall for my taste." },
			{ user = 7, ago = 25000, body = "[code].tern-tabbar { height: 24px; }[/code] in your theme CSS. Font size follows." },
			{ user = 20, ago = 24000, body = "Thanks, much better!" },
		} },

		-- Feature Requests
		{ id = 53, forum = 16, title = "Request: broadcast input to all panes", views = 1340, posts = {
			{ user = 11, ago = 9000, body = "Like tmux synchronize-panes. Please!" },
			{ user = 2, ago = 8800, body = "It exists: [code]pane.broadcast(true)[/code]. Not in the UI though." },
			{ user = 11, ago = 8500, body = "Then the request is: put it in the UI :p" },
			{ user = 16, ago = 140, body = "+1 for a menu item" },
		} },
		{ id = 54, forum = 16, title = "Request: sixel / image support", icon = "question", views = 2240, posts = {
			{ user = 19, ago = 280000, body = "Sixel please. My VT340 had it in 1987." },
			{ user = 8, ago = 279000, body = "Kitty graphics protocol is better, but +1 for images in general." },
			{ user = 1, ago = 278000, body = "Both are on the list. Kitty first, sixel after." },
		} },
		{ id = 55, forum = 16, title = "Request: per-pane opacity", views = 290, posts = {
			{ user = 20, ago = 12000, body = "Would love to fade inactive panes slightly." },
			{ user = 7, ago = 11500, body = "Theme CSS can do this: [code].tern-pane:not([data-focused]) { opacity: .8; }[/code]" },
		} },

		-- The Lounge
		{ id = 56, forum = 19, title = "What are you listening to right now?", icon = "talking", views = 2900, posts = {
			{ user = 12, ago = 30000, body = "Italo disco, as always." },
			{ user = 6, ago = 29000, body = "The hum of a flyback transformer." },
			{ user = 9, ago = 600, body = "Rain sounds and keyboard clacks." },
			{ user = 10, ago = 220, body = "Lo-fi beats to segfault to" },
		} },
		{ id = 57, forum = 19, title = "Coffee or tea while coding?", views = 1100, posts = {
			{ user = 3, ago = 150000, body = "Tea. Builder's tea. Milk, two sugars." },
			{ user = 10, ago = 149000, body = "Coffee, black, like my terminal." },
			{ user = 17, ago = 148000, body = "Sauna, then coffee." },
		} },
		{ id = 58, forum = 19, title = "Count to 1000 (no mods allowed!)", icon = "wink", views = 3400, posts = {
			{ user = 12, ago = 600000, body = "1" },
			{ user = 6, ago = 599000, body = "2" },
			{ user = 3, ago = 598000, body = "3 (sorry)" },
			{ user = 12, ago = 597000, body = "[quote=PaneInTheGlass]3 (sorry)[/quote]\nRESTART. 1" },
		} },

		-- Battlestations & Hardware
		{ id = 59, forum = 20, title = "Post your Tern setup (2026 edition)", icon = "cool", views = 12400, rating = 4.7, votes = 19, posts = {
			{ user = 6, ago = 40000, body = "New year, new thread. Monitors, keyboards, theme, layout. Go!\n\nMe: 2x 27\" 4K, green phosphor theme, 3 tabs x 4 panes. :cool:" },
			{ user = 9, ago = 39000, body = "Single ultrawide, custom 40% keyboard, Solarized-ish theme from the Themes forum." },
			{ user = 7, ago = 37000, body = "MacBook only. Foxglow theme, Reel open in a second tab all day." },
			{ user = 19, ago = 35000, body = "A real VT220 next to a 4K monitor running Tern with a VT220 theme. Irony noted." },
			{ user = 12, ago = 33000, body = "[quote=vt220fan]A real VT220 next to a 4K monitor[/quote]\nPics or it didn't happen!" },
			{ user = 19, ago = 32000, body = "Pics in the Retro forum thread :p" },
			{ user = 8, ago = 30000, body = "Linux tower, 3 monitors, one is vertical just for logs." },
			{ user = 11, ago = 28000, body = "Laptop on a stand + split keyboard. Zsh in everything." },
			{ user = 17, ago = 26000, body = "Thinkpad, 14\". Tiling WM, Tern fullscreen. Minimalism." },
			{ user = 13, ago = 24000, body = "Two monitors: left is Reel, right is everything else." },
			{ user = 10, ago = 21000, body = "Dorm desk, one 24\" monitor, my laptop, a mango :p" },
			{ user = 3, ago = 18000, body = "iPad on the left as a status board via the iOS app. Underrated." },
			{ user = 20, ago = 15000, body = "Light theme on a matte monitor. Don't @ me." },
			{ user = 16, ago = 12500, body = "Mini PC in the home lab, Tern on the laptop attached via ternd over Tailscale." },
			{ user = 4, ago = 10000, body = "Three blocks pinned on the side: CI status, clock, todo. All Luau." },
			{ user = 2, ago = 8000, body = "40 panes. One per server. You know the one." },
			{ user = 18, ago = 6000, body = "Work laptop, too many Slack windows, Tern at the back fighting for attention." },
			{ user = 15, ago = 4000, body = "Just started! One monitor, default theme, my new clock block :D" },
			{ user = 1, ago = 2900, body = "Mac Studio, two displays, nightly builds of everything. Living dangerously." },
			{ user = 14, ago = 2100, body = "laptop and um a chair. is that a setup" },
			{ user = 6, ago = 2000, body = "[quote=newb_tim]is that a setup[/quote]\nEveryone starts there. Welcome :)" },
			{ user = 9, ago = 600, body = "Upgraded to a 4K this week, the text is so crisp at 2x." },
			{ user = 13, ago = 8, body = "This thread costs me money every year :rolleyes:" },
		} },
		{ id = 60, forum = 20, title = "Mechanical keyboard thread", icon = "talking", views = 2100, posts = {
			{ user = 9, ago = 180000, body = "Mine: hand-wired 40%, tactile switches, ink-blue keycaps." },
			{ user = 12, ago = 179000, body = "Model M from 1989. The neighbours love it." },
			{ user = 7, ago = 2500, body = "Just got a split board. Typing speed down 50%, joy up 200%." },
		} },

		-- Retro Computing
		{ id = 61, forum = 21, title = "Restored a VT220 this weekend", icon = "cool", views = 1660, rating = 5, votes = 6, posts = {
			{ user = 19, ago = 34000, body = "Recapped the power supply, cleaned the CRT. Hooked it to a USB serial adapter and it runs a shell from my Linux box. Pure joy." },
			{ user = 6, ago = 33500, body = "Amber or white phosphor?" },
			{ user = 19, ago = 33000, body = "Amber. Of course." },
			{ user = 12, ago = 32500, body = "Respect. I'm still hunting for a working LK201 keyboard." },
		} },
		{ id = 62, forum = 21, title = "Favourite DOS-era terminal programs", views = 870, posts = {
			{ user = 12, ago = 350000, body = "Telix and Telemate for me. Dialing BBSes at 2400 baud." },
			{ user = 6, ago = 349000, body = "Procomm Plus. ANSI art loading line by line :D" },
			{ user = 19, ago = 348000, body = "Kermit. Don't judge me." },
		} },

		-- Archive
		{ id = 63, forum = 22, title = "Tern 0.9 beta feedback", locked = true, views = 5300, posts = {
			{ user = 1, ago = 1400000, body = "Beta is out to registered users. Feedback here please!" },
			{ user = 12, ago = 1399000, body = "Ligatures render twice on scroll. Otherwise wonderful." },
			{ user = 6, ago = 1398000, body = "Crashes when I paste 50MB. Yes, I tried." },
			{ user = 1, ago = 1300000, body = "All fixed in 1.0. Locking, thanks everyone!" },
		} },
		{ id = 64, forum = 22, title = "Old forum migration issues", locked = true, views = 1200, posts = {
			{ user = 2, ago = 1100000, body = "Post here if your account didn't migrate from the old mailing list archive." },
			{ user = 12, ago = 1099000, body = "My signature lost its formatting." },
			{ user = 2, ago = 1098000, body = "Fixed. Locking this as migration is complete." },
		} },
	},
}
