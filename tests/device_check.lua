-- Run ON the Kindle, inside KOReader's own runtime, against the installed
-- plugin. Verifies slot selection with the real Screen object.
package.path = "./?.lua;./frontend/?.lua;./plugins/?.lua;" .. package.path
package.cpath = "./libs/?.so;" .. package.cpath
require("setupkoenv")
package.path = "/mnt/us/koreader/plugins/shanshui.koplugin/?.lua;" .. package.path

local fails, checks = 0, 0
local function check(name, cond, detail)
    checks = checks + 1
    if cond then print("  ok   " .. name)
    else fails = fails + 1; print("  FAIL " .. name .. (detail and (" -- " .. tostring(detail)) or "")) end
end

-- Globals must exist before `device` is required: its module chain reads them.
G_reader_settings = require("luasettings"):open("/mnt/us/koreader/settings/reader.lua", true)
G_defaults = require("luasettings"):open("./defaults.lua", true)

-- A second KOReader process has no fully initialised Device, so the font
-- scanner trips over a missing Device method. The plugin logic under test does
-- not need font enumeration.
package.preload["fontlist"] = function()
    return { getFontList = function() return {} end, fontnames = {} }
end
package.preload["ui/font"] = function()
    return { getFace = function() return { orig_font = "cfont", size = 15 } end }
end

local Dev = require("device")
if not Dev.isKindle then Dev.isKindle = function() return true end end
if not Dev.isKobo then Dev.isKobo = function() return false end end

local Screen = Dev.screen
print(string.format("screen: %dx%d rotation=%d",
    Screen:getWidth(), Screen:getHeight(), Screen:getRotationMode()))

local ShanShui = require("main")
local s = setmetatable({}, { __index = ShanShui })
s.settings = require("luasettings"):open("/mnt/us/koreader/settings/shanshui_settings.lua", true)

local DIR = "/mnt/us/koreader/screensavers/shanshui"
s.getOutputDir = function() return DIR end

print("== slots on disk ==")
check("portrait.png exists", s:getSlotPath(false) ~= nil, s:getSlotPath(false))
check("landscape.png exists", s:getSlotPath(true) ~= nil, s:getSlotPath(true))
check("nothing missing", s:getMissingSlots() == "", s:getMissingSlots())

print("== formation detection with the real Screen ==")
local real_rot = Screen.getRotationMode
local real_w, real_h = Screen.getWidth, Screen.getHeight

-- Portrait formation
Screen.getRotationMode = function() return 0 end
Screen.getWidth = function() return 1248 end
Screen.getHeight = function() return 3296 end
check("upright -> portrait formation", s:isScreenFormationLandscape() == false)
check("upright picks portrait.png", s:getLandscapeForSleep() == DIR .. "/portrait.png",
    s:getLandscapeForSleep())

-- Landscape formation
Screen.getRotationMode = function() return 1 end
Screen.getWidth = function() return 3296 end
Screen.getHeight = function() return 1248 end
check("rotated -> landscape formation", s:isScreenFormationLandscape() == true)
check("rotated picks landscape.png", s:getLandscapeForSleep() == DIR .. "/landscape.png",
    s:getLandscapeForSleep())

-- Upside-down (rotation 2) is still portrait
Screen.getRotationMode = function() return 2 end
Screen.getWidth = function() return 1248 end
Screen.getHeight = function() return 3296 end
check("upside-down -> portrait formation", s:isScreenFormationLandscape() == false)
check("upside-down picks portrait.png", s:getLandscapeForSleep() == DIR .. "/portrait.png",
    s:getLandscapeForSleep())

-- Counter-clockwise landscape (rotation 3)
Screen.getRotationMode = function() return 3 end
Screen.getWidth = function() return 3296 end
Screen.getHeight = function() return 1248 end
check("ccw landscape -> landscape formation", s:isScreenFormationLandscape() == true)
check("ccw picks landscape.png", s:getLandscapeForSleep() == DIR .. "/landscape.png",
    s:getLandscapeForSleep())

print("== image dimensions match the slot ==")
local function png_size(path)
    local f = io.open(path, "rb"); if not f then return nil end
    local h = f:read(24); f:close()
    if not h or #h < 24 then return nil end
    local w = h:byte(17)*16777216 + h:byte(18)*65536 + h:byte(19)*256 + h:byte(20)
    local hh = h:byte(21)*16777216 + h:byte(22)*65536 + h:byte(23)*256 + h:byte(24)
    return w, hh
end
local pw, ph = png_size(DIR .. "/portrait.png")
local lw, lh = png_size(DIR .. "/landscape.png")
print(string.format("  portrait.png %dx%d, landscape.png %dx%d", pw or 0, ph or 0, lw or 0, lh or 0))
check("portrait.png is portrait", pw and ph and ph > pw)
check("landscape.png is landscape", lw and lh and lw > lh)
check("portrait.png matches screen aspect", pw == 1236 and ph == 1648,
    string.format("%dx%d", pw or 0, ph or 0))
check("landscape.png matches screen aspect", lw == 1648 and lh == 1236,
    string.format("%dx%d", lw or 0, lh or 0))

print("== isImageLandscape reads the slot correctly ==")
check("isImageLandscape(portrait.png) == false", s:isImageLandscape(DIR .. "/portrait.png") == false)
check("isImageLandscape(landscape.png) == true", s:isImageLandscape(DIR .. "/landscape.png") == true)

Screen.getRotationMode = real_rot
Screen.getWidth = real_w
Screen.getHeight = real_h

print(string.format("\n%d checks, %d failed", checks, fails))
os.exit(fails == 0 and 0 or 1)
