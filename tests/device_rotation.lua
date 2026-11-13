-- Simulate a rotation change on the device: verify onSetRotationMode refreshes
-- the slot for the formation just entered, and that the next sleep prints it.
package.path = "./?.lua;./frontend/?.lua;./plugins/?.lua;" .. package.path
package.cpath = "./libs/?.so;" .. package.cpath
require("setupkoenv")
package.path = "/mnt/us/koreader/plugins/shanshui.koplugin/?.lua;" .. package.path

G_reader_settings = require("luasettings"):open("/mnt/us/koreader/settings/reader.lua", true)
G_defaults = require("luasettings"):open("./defaults.lua", true)

package.preload["fontlist"] = function()
    return { getFontList = function() return {} end, fontnames = {} }
end
package.preload["ui/font"] = function()
    return { getFace = function() return { orig_font = "cfont", size = 15 } end }
end

local fails, checks = 0, 0
local function check(name, cond, detail)
    checks = checks + 1
    if cond then print("  ok   " .. name)
    else fails = fails + 1; print("  FAIL " .. name .. (detail and (" -- " .. tostring(detail)) or "")) end
end

local ShanShui = require("main")
local s = setmetatable({}, { __index = ShanShui })
s.settings = require("luasettings"):open("/mnt/us/koreader/settings/shanshui_settings.lua", true)
s.settings:saveSetting("orientation", "auto")
s.settings:saveSetting("enabled", true)

local DIR = "/mnt/us/koreader/screensavers/shanshui"
s.getOutputDir = function() return DIR end

-- Capture what would be spawned instead of running it.
local spawned = {}
s.isGeneratorRunning = function() return false end
local real_execute = os.execute
os.execute = function(cmd) spawned[#spawned + 1] = cmd; return 0 end

-- scheduleIn defers; run the callback immediately for this test.
local UIManager = require("ui/uimanager")
local real_schedule = UIManager.scheduleIn
UIManager.scheduleIn = function(_, _, fn) fn() end

local function modes_of(cmd)
    return cmd and cmd:match('"([%a,]+)"%s*>')
end

print("== rotation into landscape refreshes the landscape slot ==")
spawned = {}
s:onSetRotationMode(1)   -- DEVICE_ROTATED_CLOCKWISE
check("one generator run", #spawned == 1, #spawned .. " spawn(s)")
check("targets the landscape slot", modes_of(spawned[1]) == "landscape", modes_of(spawned[1]))

print("== rotation back to portrait refreshes the portrait slot ==")
spawned = {}
s:onSetRotationMode(0)   -- DEVICE_ROTATED_UPRIGHT
check("one generator run", #spawned == 1, #spawned .. " spawn(s)")
check("targets the portrait slot", modes_of(spawned[1]) == "portrait", modes_of(spawned[1]))

print("== upside-down and ccw map to the right formation ==")
spawned = {}
s:onSetRotationMode(2)   -- DEVICE_ROTATED_UPSIDE_DOWN (still portrait)
check("upside-down refreshes portrait", modes_of(spawned[1]) == "portrait", modes_of(spawned[1]))
spawned = {}
s:onSetRotationMode(3)   -- DEVICE_ROTATED_COUNTER_CLOCKWISE (landscape)
check("ccw refreshes landscape", modes_of(spawned[1]) == "landscape", modes_of(spawned[1]))

print("== a missing slot is filled ==")
-- Simulate a missing slot by stubbing the lookup (renaming on the device's
-- FAT partition is unreliable). generateNextInBackground() decides what to
-- render from getMissingSlots() when no explicit slot is requested.
local real_missing = s.getMissingSlots
s.getMissingSlots = function() return "landscape" end
spawned = {}
s:generateNextInBackground()
check("fills only the missing slot", #spawned == 1 and modes_of(spawned[1]) == "landscape",
    modes_of(spawned[1]))
s.getMissingSlots = real_missing

print("== wake-up refreshes the slot that was printed ==")
-- The printed slot lives at module scope (shared across plugin instances),
-- so drive it through the screensaver hook rather than an instance field.
-- ui/screensaver pulls in the document stack; stub the surface the plugin
-- patches so hookScreensaver() can run without it.
local Screensaver = {
    setup = function() end,
    show = function() end,
    modeExpectsPortrait = function() return true end,
}
package.preload["ui/screensaver"] = function() return Screensaver end
package.loaded["ui/screensaver"] = Screensaver
local Screen = require("device").screen
local real_rot = Screen.getRotationMode
local real_slot = s.getSlotPath

local function print_slot(is_landscape)
    Screen.getRotationMode = function() return is_landscape and 1 or 0 end
    s.getSlotPath = function(self, want_ls)
        if want_ls == is_landscape then return DIR .. (is_landscape and "/landscape.png" or "/portrait.png") end
        return nil
    end
    local fake_ss = {}
    s:hookScreensaver()
    Screensaver.setup(fake_ss, "suspend")
    s.getSlotPath = real_slot
end

print_slot(true)
spawned = {}
s:onResume()
check("wake-up after landscape print refreshes landscape",
    #spawned == 1 and modes_of(spawned[1]) == "landscape", modes_of(spawned[1]))

print_slot(false)
spawned = {}
s:onResume()
check("wake-up after portrait print refreshes portrait",
    #spawned == 1 and modes_of(spawned[1]) == "portrait", modes_of(spawned[1]))

Screen.getRotationMode = real_rot

os.execute = real_execute
UIManager.scheduleIn = real_schedule

print(string.format("\n%d checks, %d failed", checks, fails))
os.exit(fails == 0 and 0 or 1)
