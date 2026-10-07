//! Icon names: the icons Tern ships ([`Icon`]) apart from the app icons
//! (`app-vim`, `app-python`, …), which follow Tern's catalog of programs.
//! Those, names newer than the SDK and aliases go through [`Icon::Other`]:
//! `Icon::from("app-vim")`.

use super::types::string_enum;

string_enum!(
	/// A named icon from Tern's icon set.
	Icon {
		/// `logo`.
		Logo = "logo",
		/// `tern`.
		Tern = "tern",
		/// `pi-mark`.
		PiMark = "pi-mark",
		/// `arrow-up`.
		ArrowUp = "arrow-up",
		/// `arrow-down`.
		ArrowDown = "arrow-down",
		/// `arrow-left`.
		ArrowLeft = "arrow-left",
		/// `arrow-right`.
		ArrowRight = "arrow-right",
		/// `back`.
		Back = "back",
		/// `forward`.
		Forward = "forward",
		/// `chev`.
		Chev = "chev",
		/// `chev-r`.
		ChevR = "chev-r",
		/// `chev-up`.
		ChevUp = "chev-up",
		/// `corner-down-right`.
		CornerDownRight = "corner-down-right",
		/// `expand`.
		Expand = "expand",
		/// `shrink`.
		Shrink = "shrink",
		/// `minimize`.
		Minimize = "minimize",
		/// `fit`.
		Fit = "fit",
		/// `zoom-in`.
		ZoomIn = "zoom-in",
		/// `zoom-out`.
		ZoomOut = "zoom-out",
		/// `more`.
		More = "more",
		/// `grip`.
		Grip = "grip",
		/// `compass`.
		Compass = "compass",
		/// `check`.
		Check = "check",
		/// `x`.
		X = "x",
		/// `x-circle`.
		XCircle = "x-circle",
		/// `warn`.
		Warn = "warn",
		/// `info`.
		Info = "info",
		/// `help`.
		Help = "help",
		/// `bell`.
		Bell = "bell",
		/// `shield`.
		Shield = "shield",
		/// `shield-alert`.
		ShieldAlert = "shield-alert",
		/// `slash`.
		Slash = "slash",
		/// `slash-circle`.
		SlashCircle = "slash-circle",
		/// `lock`.
		Lock = "lock",
		/// `key`.
		Key = "key",
		/// `key-round`.
		KeyRound = "key-round",
		/// `plus`.
		Plus = "plus",
		/// `minus`.
		Minus = "minus",
		/// `clock`.
		Clock = "clock",
		/// `timer`.
		Timer = "timer",
		/// `doc`.
		Doc = "doc",
		/// `file`.
		File = "file",
		/// `file-code`.
		FileCode = "file-code",
		/// `file-image`.
		FileImage = "file-image",
		/// `file-pdf`.
		FilePdf = "file-pdf",
		/// `file-plus`.
		FilePlus = "file-plus",
		/// `files`.
		Files = "files",
		/// `folder`.
		Folder = "folder",
		/// `folder-go`.
		FolderGo = "folder-go",
		/// `folder-minus`.
		FolderMinus = "folder-minus",
		/// `folder-open`.
		FolderOpen = "folder-open",
		/// `folder-plus`.
		FolderPlus = "folder-plus",
		/// `markdown`.
		Markdown = "markdown",
		/// `note`.
		Note = "note",
		/// `clipboard`.
		Clipboard = "clipboard",
		/// `save`.
		Save = "save",
		/// `inbox`.
		Inbox = "inbox",
		/// `image`.
		Image = "image",
		/// `newspaper`.
		Newspaper = "newspaper",
		/// `trash`.
		Trash = "trash",
		/// `branch`.
		Branch = "branch",
		/// `commit`.
		Commit = "commit",
		/// `merge`.
		Merge = "merge",
		/// `rebase`.
		Rebase = "rebase",
		/// `pull`.
		Pull = "pull",
		/// `push`.
		Push = "push",
		/// `fetch`.
		Fetch = "fetch",
		/// `stash`.
		Stash = "stash",
		/// `cherry`.
		Cherry = "cherry",
		/// `diff`.
		Diff = "diff",
		/// `unified`.
		Unified = "unified",
		/// `split`.
		Split = "split",
		/// `split-down`.
		SplitDown = "split-down",
		/// `revert`.
		Revert = "revert",
		/// `history`.
		History = "history",
		/// `play`.
		Play = "play",
		/// `pause`.
		Pause = "pause",
		/// `stop`.
		Stop = "stop",
		/// `rewind`.
		Rewind = "rewind",
		/// `fast-forward`.
		FastForward = "fast-forward",
		/// `frame-next`.
		FrameNext = "frame-next",
		/// `frame-prev`.
		FramePrev = "frame-prev",
		/// `repeat`.
		Repeat = "repeat",
		/// `redo`.
		Redo = "redo",
		/// `undo`.
		Undo = "undo",
		/// `run`.
		Run = "run",
		/// `reel`.
		Reel = "reel",
		/// `mic`.
		Mic = "mic",
		/// `wave`.
		Wave = "wave",
		/// `vibrate`.
		Vibrate = "vibrate",
		/// `power`.
		Power = "power",
		/// `send`.
		Send = "send",
		/// `share`.
		Share = "share",
		/// `open`.
		Open = "open",
		/// `download`.
		Download = "download",
		/// `copy`.
		Copy = "copy",
		/// `scissors`.
		Scissors = "scissors",
		/// `bold`.
		Bold = "bold",
		/// `italic`.
		Italic = "italic",
		/// `underline`.
		Underline = "underline",
		/// `type`.
		Type = "type",
		/// `align-left`.
		AlignLeft = "align-left",
		/// `align-center`.
		AlignCenter = "align-center",
		/// `align-right`.
		AlignRight = "align-right",
		/// `align-justify`.
		AlignJustify = "align-justify",
		/// `pen`.
		Pen = "pen",
		/// `eraser`.
		Eraser = "eraser",
		/// `wand`.
		Wand = "wand",
		/// `cursor`.
		Cursor = "cursor",
		/// `selection`.
		Selection = "selection",
		/// `path-insert`.
		PathInsert = "path-insert",
		/// `prompt`.
		Prompt = "prompt",
		/// `vector`.
		Vector = "vector",
		/// `blur`.
		Blur = "blur",
		/// `ink`.
		Ink = "ink",
		/// `palette`.
		Palette = "palette",
		/// `columns`.
		Columns = "columns",
		/// `grid`.
		Grid = "grid",
		/// `sidebar`.
		Sidebar = "sidebar",
		/// `tabs-h`.
		TabsH = "tabs-h",
		/// `tabs-v`.
		TabsV = "tabs-v",
		/// `dock-up`.
		DockUp = "dock-up",
		/// `dock-down`.
		DockDown = "dock-down",
		/// `dock-left`.
		DockLeft = "dock-left",
		/// `dock-right`.
		DockRight = "dock-right",
		/// `pip`.
		Pip = "pip",
		/// `pip-tl`.
		PipTl = "pip-tl",
		/// `pip-tr`.
		PipTr = "pip-tr",
		/// `pip-bl`.
		PipBl = "pip-bl",
		/// `pip-exit`.
		PipExit = "pip-exit",
		/// `layers`.
		Layers = "layers",
		/// `stack`.
		Stack = "stack",
		/// `kanban`.
		Kanban = "kanban",
		/// `canvas`.
		Canvas = "canvas",
		/// `diagram`.
		Diagram = "diagram",
		/// `flow`.
		Flow = "flow",
		/// `peek`.
		Peek = "peek",
		/// `tree`.
		Tree = "tree",
		/// `box`.
		Box = "box",
		/// `eye`.
		Eye = "eye",
		/// `eye-off`.
		EyeOff = "eye-off",
		/// `terminal`.
		Terminal = "terminal",
		/// `code`.
		Code = "code",
		/// `braces`.
		Braces = "braces",
		/// `binary`.
		Binary = "binary",
		/// `bug`.
		Bug = "bug",
		/// `cpu`.
		Cpu = "cpu",
		/// `database`.
		Database = "database",
		/// `server`.
		Server = "server",
		/// `laptop`.
		Laptop = "laptop",
		/// `monitor`.
		Monitor = "monitor",
		/// `gauge`.
		Gauge = "gauge",
		/// `activity`.
		Activity = "activity",
		/// `chart`.
		Chart = "chart",
		/// `plug`.
		Plug = "plug",
		/// `puzzle`.
		Puzzle = "puzzle",
		/// `wrench`.
		Wrench = "wrench",
		/// `gear`.
		Gear = "gear",
		/// `sliders`.
		Sliders = "sliders",
		/// `stethoscope`.
		Stethoscope = "stethoscope",
		/// `hash`.
		Hash = "hash",
		/// `tag`.
		Tag = "tag",
		/// `link`.
		Link = "link",
		/// `globe`.
		Globe = "globe",
		/// `broadcast`.
		Broadcast = "broadcast",
		/// `search`.
		Search = "search",
		/// `funnel`.
		Funnel = "funnel",
		/// `funnel-x`.
		FunnelX = "funnel-x",
		/// `sort-x`.
		SortX = "sort-x",
		/// `swap`.
		Swap = "swap",
		/// `list`.
		List = "list",
		/// `list-checks`.
		ListChecks = "list-checks",
		/// `pin`.
		Pin = "pin",
		/// `keyboard`.
		Keyboard = "keyboard",
		/// `user`.
		User = "user",
		/// `users`.
		Users = "users",
		/// `message`.
		Message = "message",
		/// `brain`.
		Brain = "brain",
		/// `sparkle`.
		Sparkle = "sparkle",
		/// `lightbulb`.
		Lightbulb = "lightbulb",
		/// `bolt`.
		Bolt = "bolt",
		/// `flame`.
		Flame = "flame",
		/// `rocket`.
		Rocket = "rocket",
		/// `moon`.
		Moon = "moon",
		/// `sun`.
		Sun = "sun",
		/// `cart`.
		Cart = "cart",
		/// `footprints`.
		Footprints = "footprints",
		/// `scale`.
		Scale = "scale",
		/// `log-in`.
		LogIn = "log-in",
		/// `log-out`.
		LogOut = "log-out",
	}
);
