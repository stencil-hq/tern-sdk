--!strict
-- Helpers both halves of longrun require: reading a command line's program
-- and writing durations the way people say them.

local common = {}

-- Words that run another program: the program is the next word.
local WRAPPERS: { [string]: boolean } = {
	sudo = true,
	env = true,
	time = true,
	nice = true,
	nohup = true,
	command = true,
	exec = true,
	caffeinate = true,
}

-- Programs that stay open until you leave them. Their run time measures how
-- long you used them, not how long you waited, so they are never "slow".
common.INTERACTIVE = {
	vi = true,
	vim = true,
	nvim = true,
	hx = true,
	nano = true,
	emacs = true,
	less = true,
	more = true,
	man = true,
	ssh = true,
	mosh = true,
	tmux = true,
	screen = true,
	top = true,
	htop = true,
	btop = true,
	watch = true,
	fzf = true,
	lazygit = true,
	tig = true,
	ipython = true,
	psql = true,
	mysql = true,
	sqlite3 = true,
	["redis-cli"] = true,
	omp = true,
}

-- Interpreters that are a prompt when run bare and a job when given a
-- script: `python` waits for you, `python train.py` makes you wait.
local REPLS: { [string]: boolean } = {
	python = true,
	python3 = true,
	node = true,
	irb = true,
	lua = true,
}

-- The program a command line runs (the first word that is not a
-- `VAR=value` prefix, a wrapper or a wrapper's flag, without its
-- directory), and whether more words follow it.
function common.program(line: string): (string?, boolean)
	local at = 1
	while true do
		local from, to, word = string.find(line, "(%S+)", at)
		if from == nil then
			return nil, false
		end
		local w = word :: string
		local assignment = string.find(w, "^[%a_][%w_]*=") ~= nil
		local flag = string.sub(w, 1, 1) == "-"
		if not assignment and not flag and not WRAPPERS[w] then
			local rest = string.sub(line, (to :: number) + 1)
			return string.match(w, "([^/]+)$"), string.find(rest, "%S") ~= nil
		end
		at = (to :: number) + 1
	end
end

-- Whether `line` runs a program you use rather than wait for: one
-- `INTERACTIVE` lists, or a bare interpreter.
function common.interactive(line: string): boolean
	local program, args = common.program(line)
	if program == nil then
		return false
	end
	return common.INTERACTIVE[program] == true or (REPLS[program] == true and not args)
end

-- `ms` as "8.4s", "42s", "1m 12s" or "2h 5m".
function common.duration(ms: number): string
	if ms < 10000 then
		return string.format("%.1fs", ms / 1000)
	end
	local seconds = math.floor(ms / 1000)
	if seconds < 60 then
		return string.format("%ds", seconds)
	end
	local minutes = seconds // 60
	if minutes < 60 then
		return string.format("%dm %ds", minutes, seconds % 60)
	end
	return string.format("%dh %dm", minutes // 60, minutes % 60)
end

-- `path` with the home directory written `~`.
function common.tilde(path: string, home: string?): string
	if home and home ~= "" then
		if path == home then
			return "~"
		end
		if string.sub(path, 1, #home + 1) == home .. "/" then
			return "~" .. string.sub(path, #home + 1)
		end
	end
	return path
end

-- `text` cut to `max` characters, the last one an ellipsis.
function common.clip(text: string, max: number): string
	local length = utf8.len(text)
	if length == nil then -- not UTF-8: count bytes
		return if #text <= max then text else string.sub(text, 1, max - 1) .. "…"
	end
	if length <= max then
		return text
	end
	local cut = utf8.offset(text, max) or (#text + 1)
	return string.sub(text, 1, cut - 1) .. "…"
end

return common
