-- Verify the orientation-slot logic in main.lua without a device.
-- Run from a KOReader directory: SDL_VIDEODRIVER=dummy ./lib/koreader/luajit check.lua
package.path = "./?.lua;./frontend/?.lua;./plugins/?.lua;" .. package.path
package.cpath = "./libs/?.so;" .. package.cpath
require("setupkoenv")

local PLUGIN = os.getenv("SHANSHUI_DIR") or "/tmp/shanshui"
local OUT = os.getenv("SHANSHUI_OUT") or "/tmp/ss_out"
package.path = PLUGIN .. "/?.lua;" .. package.path

-- Stub the UI stack that needs MuPDF/freetype (glibc on this host is older
-- than KOReader's bundled libs). The plugin logic under test is untouched.
package.preload["ui/font"] = function() return { getFace = function() return {} end } end
package.preload["fontlist"] = function() return { getFontList = function() return {} end, fontnames = {} } end
package.preload["ui/widget/imageviewer"] = function() return { new = function() end } end
package.preload["ui/widget/pathchooser"] = function() return { new = function() end } end
package.preload["ui/widget/menu"] = function() return { new = function() end } end
package.preload["ui/widget/inputdialog"] = function() return { new = function() end } end

local fails, checks = 0, 0
local function check(name, cond, detail)
    checks = checks + 1
    if cond then print("  ok   " .. name)
    else fails = fails + 1; print("  FAIL " .. name .. (detail and (" — " .. tostring(detail)) or "")) end
end

G_reader_settings = require("luasettings"):open("/tmp/shan.reader.lua", true)
G_defaults = require("luasettings"):open("./defaults.lua", true)

-- Capture the screensaver hooks the plugin installs. ui/screensaver pulls in
-- the whole widget stack (booklist -> MuPDF), so stub just the surface the
-- plugin patches.
local Screensaver = {
    setup = function() end,
    show = function() end,
    modeExpectsPortrait = function() return true end,
}
package.preload["ui/screensaver"] = function() return Screensaver end

local ShanShui = require("main")
local s = setmetatable({}, { __index = ShanShui })
s.settings = require("luasettings"):open("/tmp/shan_plugin.lua", true)
s.settings:saveSetting("orientation", "auto")
s.settings:saveSetting("enabled", true)

-- Point the plugin at the fixture directory.
s.getOutputDir = function() return OUT end

print("== slot discovery ==")
check("portrait slot found", s:getSlotPath(false) == OUT .. "/portrait.png",
    s:getSlotPath(false))
check("landscape slot found", s:getSlotPath(true) == OUT .. "/landscape.png",
    s:getSlotPath(true))
check("both slots present -> nothing missing", s:getMissingSlots() == "",
    s:getMissingSlots())

print("== formation -> slot ==")
local Screen = require("device").screen
local real_rot = Screen.getRotationMode
Screen.getRotationMode = function() return 0 end
check("upright formation is portrait", s:isScreenFormationLandscape() == false)
check("portrait formation picks portrait.png",
    s:getLandscapeForSleep() == OUT .. "/portrait.png", s:getLandscapeForSleep())
Screen.getRotationMode = function() return 1 end
check("rotated formation is landscape", s:isScreenFormationLandscape() == true)
check("landscape formation picks landscape.png",
    s:getLandscapeForSleep() == OUT .. "/landscape.png", s:getLandscapeForSleep())

print("== manual orientation overrides rotation ==")
s.settings:saveSetting("orientation", "portrait")
Screen.getRotationMode = function() return 1 end
check("locked portrait ignores landscape rotation",
    s:isScreenFormationLandscape() == false)
s.settings:saveSetting("orientation", "landscape")
Screen.getRotationMode = function() return 0 end
check("locked landscape ignores portrait rotation",
    s:isScreenFormationLandscape() == true)
s.settings:saveSetting("orientation", "auto")
Screen.getRotationMode = real_rot

print("== missing slot falls back, does not crash ==")
local realExists = require("util").pathExists
local hidden = OUT .. "/landscape.png"
os.rename(hidden, hidden .. ".bak")
Screen.getRotationMode = function() return 1 end
check("missing landscape slot reported", s:getMissingSlots() == "landscape",
    s:getMissingSlots())
local picked = s:getLandscapeForSleep()
check("falls back to the other slot", picked == OUT .. "/portrait.png", picked)
os.rename(hidden .. ".bak", hidden)

print("== pool only lists generated landscapes ==")
local pool = s:getLandscapePool()
local all_pool = true
for _, p in ipairs(pool) do
    if not p:match("/landscape_[^/]*%.png$") then all_pool = false end
end
check("pool contains only landscape_*.png", all_pool and #pool >= 2, #pool .. " entries")
check("pool excludes the slots themselves",
    not table.concat(pool, " "):find("/portrait.png", 1, true))

print("== screensaver hook picks the matching slot ==")
-- The hook closes over the instance that installed it, so drive the real
-- setup() through that instance rather than a stand-in.
s:hookScreensaver()
local fake_ss = { image = nil, image_file = nil }
Screen.getRotationMode = function() return 1 end
Screensaver.setup(fake_ss, "suspend")
check("hook selected the landscape slot",
    fake_ss.image_file == OUT .. "/landscape.png", fake_ss.image_file)
check("hook marked the image landscape", fake_ss._shanshui_is_landscape == true)
check("hook marked shanshui active", fake_ss._shanshui_active == true)
Screen.getRotationMode = real_rot

print(string.format("\n%d checks, %d failed", checks, fails))
os.exit(fails == 0 and 0 or 1)
