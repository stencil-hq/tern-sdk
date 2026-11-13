-- Numbers and dates as vBulletin 3.8 prints them with its default options:
-- `1,234`, `Today` / `Yesterday` / `11-08-2026`, `03:14 PM`, `Nov 2004`.

local Fmt = {}

-- `n` with thousands separators: 12345 → "12,345".
function Fmt.num(n: number): string
	local s = tostring(math.floor(math.abs(n)))
	local out = string.reverse((string.gsub(string.reverse(s), "(%d%d%d)", "%1,")))
	if string.sub(out, 1, 1) == "," then
		out = string.sub(out, 2)
	end
	return if n < 0 then "-" .. out else out
end

local function same_day(a: number, b: number): boolean
	local x, y = os.date("*t", a), os.date("*t", b)
	return x.year == y.year and x.yday == y.yday
end

-- The day of `t`: "Today", "Yesterday" or "mm-dd-yyyy".
function Fmt.day(t: number): string
	local now = os.time()
	if same_day(t, now) then
		return "Today"
	elseif same_day(t, now - 86400) then
		return "Yesterday"
	end
	return os.date("%m-%d-%Y", t)
end

-- The clock time of `t`: "03:14 PM".
function Fmt.time(t: number): string
	return os.date("%I:%M %p", t)
end

-- "Today, 03:14 PM", as a post's header shows it.
function Fmt.stamp(t: number): string
	return Fmt.day(t) .. ", " .. Fmt.time(t)
end

-- The calendar date of `t`: "11-08-2026".
function Fmt.date(t: number): string
	return os.date("%m-%d-%Y", t)
end

-- Month and year, as Join Date shows it: "Nov 2004".
function Fmt.month(t: number): string
	return os.date("%b %Y", t)
end


-- `text` cut to `n` bytes on a character boundary, with "..." when cut.
function Fmt.clip(text: string, n: number): string
	if #text <= n then
		return text
	end
	local cut = n
	while cut > 0 do
		local b = string.byte(text, cut + 1)
		if not b or b < 0x80 or b >= 0xC0 then
			break
		end
		cut -= 1
	end
	return string.sub(text, 1, cut) .. "..."
end

return Fmt
