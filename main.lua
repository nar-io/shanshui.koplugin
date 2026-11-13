--[[--
Shan Shui Screensaver Plugin for KOReader
Procedurally generated Chinese landscape paintings for every sleep.
Based on LingDong-/shan-shui-inf procedural algorithms.
--]]--

local DataStorage = require("datastorage")
local Device = require("device")
local Screen = Device.screen
local ImageViewer = require("ui/widget/imageviewer")
local InfoMessage = require("ui/widget/infomessage")
local LuaSettings = require("luasettings")
local UIManager = require("ui/uimanager")
local WidgetContainer = require("ui/widget/container/widgetcontainer")
local logger = require("logger")
local lfs = require("libs/libkoreader-lfs")
local util = require("util")
local _ = require("gettext")

local ShanShui = WidgetContainer:extend{
    name = "shanshui",
    is_doc_only = false,
}

local PLUGIN_DIR = "/mnt/us/koreader/plugins/shanshui.koplugin"
local SCREENSAVER_DIR = "/mnt/us/koreader/screensavers/shanshui"
local SETTINGS_PATH = DataStorage:getSettingsDir() .. "/shanshui_settings.lua"

local PROFILES = {
    ["1236x1648"] = { min = 1236, max = 1648, label = "Kindle Paperwhite 5 / 11th Gen (1236x1648)" },
    ["1404x1872"] = { min = 1404, max = 1872, label = "Kindle Oasis 2/3 / Kobo Libra 2 / Clara 2E (1404x1872)" },
    ["1072x1448"] = { min = 1072, max = 1448, label = "Kindle PW 3/4 / Voyage / Oasis 1 / Clara HD (1072x1448)" },
    ["1860x2480"] = { min = 1860, max = 2480, label = "Kindle Scribe / Kobo Elipsa / Boox Note (1860x2480)" },
    ["1264x1680"] = { min = 1264, max = 1680, label = "Kobo Forma / Sage (1264x1680)" },
    ["758x1024"]  = { min = 758,  max = 1024, label = "Kindle Paperwhite 1/2 / Kobo Aura (758x1024)" },
    ["600x800"]   = { min = 600,  max = 800,  label = "Standard / Kindle Basic / Legacy Devices (600x800)" },
}

local PROFILE_ORDER = {
    "1236x1648",
    "1404x1872",
    "1072x1448",
    "1860x2480",
    "1264x1680",
    "758x1024",
    "600x800",
}

local function isScreenLandscape()
    if not Screen then return false end
    local rot = Screen:getRotationMode()
    return (rot % 2 == 1) or (Screen:getWidth() > Screen:getHeight())
end

-- Formation the screensaver will be printed in. Honours the manual
-- Orientation setting; "auto" follows the live screen rotation.
function ShanShui:isScreenFormationLandscape()
    local ori = self.settings:readSetting("orientation") or "auto"
    if ori == "landscape" then return true end
    if ori == "portrait" then return false end
    return isScreenLandscape()
end

local function getPNGDimensions(file_path)
    if not file_path then return nil, nil end
    local f = io.open(file_path, "rb")
    if not f then return nil, nil end
    local header = f:read(24)
    f:close()
    if not header or #header < 24 then return nil, nil end
    if header:sub(1, 8) ~= "\137PNG\r\n\026\n" then return nil, nil end
    local w = header:byte(17) * 16777216 + header:byte(18) * 65536 + header:byte(19) * 256 + header:byte(20)
    local h = header:byte(21) * 16777216 + header:byte(22) * 65536 + header:byte(23) * 256 + header:byte(24)
    return w, h
end

function ShanShui:detectDeviceProfile()
    local w = Screen and Screen:getWidth() or 0
    local h = Screen and Screen:getHeight() or 0
    if w <= 0 or h <= 0 then
        return 600, 800, "600x800", _("Standard (600x800)")
    end
    local min_dim = math.min(w, h)
    local max_dim = math.max(w, h)
    local key = string.format("%dx%d", min_dim, max_dim)
    if PROFILES[key] then
        return min_dim, max_dim, key, PROFILES[key].label
    end
    if min_dim >= 400 and max_dim >= 600 then
        return min_dim, max_dim, key, string.format(_("Custom Screen (%dx%d)"), min_dim, max_dim)
    end
    return 600, 800, "600x800", _("Standard (600x800)")
end

function ShanShui:getTargetDimensions()
    local res_setting = self.settings:readSetting("resolution") or "auto"
    local min_dim, max_dim
    if res_setting == "auto" then
        min_dim, max_dim = self:detectDeviceProfile()
    elseif PROFILES[res_setting] then
        min_dim = PROFILES[res_setting].min
        max_dim = PROFILES[res_setting].max
    else
        min_dim, max_dim = 600, 800
    end

    local ori = self.settings:readSetting("orientation") or "auto"
    local is_ls
    if ori == "portrait" then
        is_ls = false
    elseif ori == "landscape" then
        is_ls = true
    else
        is_ls = isScreenLandscape()
    end

    if is_ls then
        return max_dim, min_dim
    else
        return min_dim, max_dim
    end
end

function ShanShui:init()
    self.settings = LuaSettings:open(SETTINGS_PATH)
    if self.settings:readSetting("enabled") == nil then
        self.settings:saveSetting("enabled", true)
    end
    if self.settings:readSetting("orientation") == nil then
        self.settings:saveSetting("orientation", "auto")
    end
    if self.settings:readSetting("transparent_sky") == nil then
        self.settings:saveSetting("transparent_sky", false)
    end
    if self.settings:readSetting("resolution") == nil then
        self.settings:saveSetting("resolution", "auto")
    end
    self.settings:flush()

    self.need_refresh_on_resume = false
    self:hookScreensaver()
    self.ui.menu:registerToMainMenu(self)

    UIManager:nextTick(function()
        -- First launch (or after an upgrade from 1.0.x): make sure both
        -- formations have a picture, so the very first sleep already prints
        -- the right one.
        if self:getMissingSlots() ~= "" then
            logger.info("ShanShui: missing orientation slot(s), triggering initial background generation")
            self:generateNextInBackground()
        end
    end)
end

function ShanShui:isEnabled()
    return self.settings:readSetting("enabled") == true
end

function ShanShui:isTransparentSky()
    return self.settings:readSetting("transparent_sky") == true
end

function ShanShui:getOutputDir()
    util.makePath(SCREENSAVER_DIR)
    return SCREENSAVER_DIR
end

function ShanShui:isImageLandscape(img_path)
    if img_path then
        local w, h = getPNGDimensions(img_path)
        if w and h and w ~= h then
            return w > h
        end
    end
    local ori = self.settings:readSetting("orientation") or "auto"
    if ori == "landscape" then
        return true
    elseif ori == "portrait" then
        return false
    else
        return isScreenLandscape()
    end
end

function ShanShui:getCurrentLandscape()
    -- Preferred slot for the current screen formation, then the other one.
    local out_dir = self:getOutputDir()
    local want_ls = self:isScreenFormationLandscape()
    local order = want_ls and { "landscape", "portrait" } or { "portrait", "landscape" }
    for _i, suffix in ipairs(order) do
        local path = out_dir .. "/" .. suffix .. ".png"
        if util.pathExists(path) then
            return path
        end
    end

    -- Legacy single-slot file from 1.0.x, and the pool as a last resort.
    local legacy = out_dir .. "/current.png"
    if util.pathExists(legacy) then
        return legacy
    end

    local pool = self:getLandscapePool()
    if #pool > 0 then
        return pool[1]
    end

    return nil
end

-- Path of the slot that matches `is_landscape`, or nil when it is missing.
function ShanShui:getSlotPath(is_landscape)
    local suffix = is_landscape and "landscape" or "portrait"
    local path = self:getOutputDir() .. "/" .. suffix .. ".png"
    if util.pathExists(path) then
        return path
    end
    return nil
end

-- Slots that still need a picture, as a comma-separated generator argument.
function ShanShui:getMissingSlots()
    local missing = {}
    if not self:getSlotPath(false) then missing[#missing + 1] = "portrait" end
    if not self:getSlotPath(true) then missing[#missing + 1] = "landscape" end
    return table.concat(missing, ",")
end

function ShanShui:getLandscapePool()
    local out_dir = self:getOutputDir()
    local files = {}
    for entry in lfs.dir(out_dir) do
        if entry:match("^landscape_.*%.png$") then
            table.insert(files, out_dir .. "/" .. entry)
        end
    end
    table.sort(files, function(a, b) return a > b end)
    return files
end

function ShanShui:getLandscapeForSleep()
    -- The picture has to match the formation the device is in *now*: the
    -- previous code picked whatever orientation had been generated last,
    -- which is why a rotation followed by an immediate sleep printed the
    -- old formation. Each formation has its own slot, so read that slot.
    local want_landscape = self:isScreenFormationLandscape()

    self.need_refresh_on_resume = true

    local slot = self:getSlotPath(want_landscape)
    if slot then
        return slot
    end

    -- The slot for this formation does not exist yet (first run, or the
    -- background generator has not finished). Fall back to the other slot so
    -- the screensaver is never blank, and let onResume() fill the gap.
    logger.info("ShanShui: no " .. (want_landscape and "landscape" or "portrait")
        .. " slot yet, falling back")
    local fallback = self:getSlotPath(not want_landscape)
    if fallback then
        return fallback
    end

    local pool = self:getLandscapePool()
    return pool[1]
end

function ShanShui:isGeneratorRunning()
    local pid_file = "/tmp/shanshui_generator.pid"
    if util.pathExists(pid_file) then
        local f = io.open(pid_file, "r")
        if f then
            local pid = f:read("*l")
            f:close()
            if pid and pid ~= "" then
                local res = os.execute(string.format("kill -0 %s 2>/dev/null", pid))
                if res == 0 or res == true then
                    return true
                end
            end
        end
    end
    return false
end

function ShanShui:generateNextInBackground(force)
    if not self:isEnabled() and not force then
        return false
    end

    if self:isGeneratorRunning() then
        logger.dbg("ShanShui: Generator is already actively running")
        return false
    end

    local out_dir = self:getOutputDir()
    local script = PLUGIN_DIR .. "/generator.sh"
    if not util.pathExists(script) then
        logger.warn("ShanShui: Generator script not found at", script)
        return false
    end

    local width, height = self:getTargetDimensions()
    local trans_arg = self:isTransparentSky() and "1" or "0"
    -- A forced run (settings change) refreshes both formations; an automatic
    -- one only fills the slots that are still empty.
    local modes = force and "portrait,landscape" or self:getMissingSlots()
    if modes == "" then
        logger.dbg("ShanShui: both orientation slots are present, nothing to generate")
        return false
    end
    local cmd = string.format("chmod +x %q && nice -n 19 /bin/sh %q %q %d %d %s %q >/dev/null 2>&1 &",
        script, script, out_dir, width, height, trans_arg, modes)
    logger.info(string.format("ShanShui: Spawning background generator (%dx%d, trans=%s, modes=%s): %s",
        width, height, trans_arg, modes, cmd))
    os.execute(cmd)
    return true
end

function ShanShui:hookScreensaver()
    local Screensaver = require("ui/screensaver")
    if Screensaver._shanshui_hooked then
        return
    end
    Screensaver._shanshui_hooked = true

    local orig_setup = Screensaver.setup
    local orig_show = Screensaver.show
    local orig_modeExpectsPortrait = Screensaver.modeExpectsPortrait
    local plugin_self = self

    Screensaver.setup = function(ss_self, event, event_message)
        orig_setup(ss_self, event, event_message)
        ss_self._shanshui_active = false
        if plugin_self:isEnabled() then
            local img = plugin_self:getLandscapeForSleep()
            if img and util.pathExists(img) then
                ss_self.image = nil
                ss_self.image_file = img
                ss_self.screensaver_type = "random_image"
                if not plugin_self:isTransparentSky() then
                    ss_self.screensaver_background = "none"
                end
                ss_self._shanshui_active = true
                ss_self._shanshui_is_landscape = plugin_self:isImageLandscape(img)
                logger.info("ShanShui: screensaver active with image:", img,
                    "is_landscape:", ss_self._shanshui_is_landscape,
                    "screen_landscape:", plugin_self:isScreenFormationLandscape(),
                    "transparent_sky:", plugin_self:isTransparentSky())
            end
        end
    end

    Screensaver.modeExpectsPortrait = function(ss_self)
        if plugin_self:isEnabled() and ss_self._shanshui_active then
            if ss_self._shanshui_is_landscape then
                return false
            else
                return true
            end
        end
        return orig_modeExpectsPortrait(ss_self)
    end

    Screensaver.show = function(ss_self)
        -- The slot chosen in setup() already matches the current formation, so
        -- the picture is printed as-is. Rotating the framebuffer here was what
        -- made the *previous* formation's picture come out sideways: the
        -- rotation was applied after the image had been picked, and it is not
        -- undone until the next wake-up.
        orig_show(ss_self)
    end
end

function ShanShui:onSetRotationMode(rotation)
    logger.dbg("ShanShui: onSetRotationMode", rotation)
    if self:isEnabled() and (self.settings:readSetting("orientation") or "auto") == "auto" then
        -- Both formations get their own slot, so a rotation no longer needs a
        -- fresh render before the next sleep can print the right picture.
        -- Only fill in what is missing.
        if self:getMissingSlots() ~= "" then
            logger.info("ShanShui: Screen rotation changed, ensuring both orientation slots exist")
            UIManager:scheduleIn(2, function()
                self:generateNextInBackground()
            end)
        end
    end
end

function ShanShui:onSuspend()
    logger.dbg("ShanShui: onSuspend")
    if self:isEnabled() then
        self.need_refresh_on_resume = true
    end
end

function ShanShui:onResume()
    logger.dbg("ShanShui: onResume")
    if not self:isEnabled() then return end
    -- Wake-up is the safe moment to regenerate: the screensaver is gone, so
    -- rewriting a slot cannot race with a print. Fill whichever formation is
    -- missing (the one that was just used is now free to be replaced).
    self.need_refresh_on_resume = false
    UIManager:scheduleIn(2, function()
        self:generateNextInBackground()
    end)
end

function ShanShui:addToMainMenu(menu_items)
    local _min, _max, detected_key, detected_label = self:detectDeviceProfile()
    local res_subitems = {
        {
            text = string.format(_("Auto (Device: %s)"), detected_label),
            checked_func = function()
                return (self.settings:readSetting("resolution") or "auto") == "auto"
            end,
            callback = function()
                self.settings:saveSetting("resolution", "auto")
                self.settings:flush()
                self:generateNextInBackground(true)
            end,
        },
    }

    for _idx, key in ipairs(PROFILE_ORDER) do
        local prof = PROFILES[key]
        table.insert(res_subitems, {
            text = prof.label,
            checked_func = function()
                return self.settings:readSetting("resolution") == key
            end,
            callback = function()
                self.settings:saveSetting("resolution", key)
                self.settings:flush()
                self:generateNextInBackground(true)
            end,
        })
    end

    menu_items.shanshui = {
        text = _("Shan Shui Screensaver"),
        sorting_hint = "more_tools",
        sub_item_table = {
            {
                text = _("Enable"),
                checked_func = function()
                    return self:isEnabled()
                end,
                callback = function()
                    local state = not self:isEnabled()
                    self.settings:saveSetting("enabled", state)
                    self.settings:flush()
                    if state and not self:getCurrentLandscape() then
                        self:generateNextInBackground()
                    end
                end,
            },
            {
                text = _("Transparent Sky"),
                checked_func = function()
                    return self:isTransparentSky()
                end,
                callback = function()
                    local new_state = not self:isTransparentSky()
                    self.settings:saveSetting("transparent_sky", new_state)
                    self.settings:flush()
                    UIManager:show(InfoMessage:new{
                        text = new_state
                            and _("Transparent sky enabled. Only the sky above mountain silhouettes will be transparent.")
                            or _("Transparent sky disabled. Landscapes will be rendered with opaque white sky."),
                    })
                    self:generateNextInBackground(true)
                end,
            },
            {
                text = _("Resolution & Device"),
                sub_item_table = res_subitems,
            },
            {
                text = _("Orientation"),
                sub_item_table = {
                    {
                        text = _("Auto (Follow Screen)"),
                        checked_func = function()
                            return self.settings:readSetting("orientation") == "auto"
                        end,
                        callback = function()
                            self.settings:saveSetting("orientation", "auto")
                            self.settings:flush()
                            self:generateNextInBackground(true)
                        end,
                    },
                    {
                        text = _("Portrait"),
                        checked_func = function()
                            return self.settings:readSetting("orientation") == "portrait"
                        end,
                        callback = function()
                            self.settings:saveSetting("orientation", "portrait")
                            self.settings:flush()
                            self:generateNextInBackground(true)
                        end,
                    },
                    {
                        text = _("Landscape"),
                        checked_func = function()
                            return self.settings:readSetting("orientation") == "landscape"
                        end,
                        callback = function()
                            self.settings:saveSetting("orientation", "landscape")
                            self.settings:flush()
                            self:generateNextInBackground(true)
                        end,
                    },
                },
            },
            {
                text = _("View Current Landscape"),
                enabled_func = function()
                    return self:getCurrentLandscape() ~= nil
                end,
                callback = function()
                    local img = self:getCurrentLandscape()
                    if img and util.pathExists(img) then
                        UIManager:show(ImageViewer:new{
                            file = img,
                        })
                    else
                        UIManager:show(InfoMessage:new{
                            text = _("No generated landscape found yet."),
                        })
                    end
                end,
            },
            {
                text = _("Generate New Landscape Now"),
                callback = function()
                    if self:isGeneratorRunning() then
                        UIManager:show(InfoMessage:new{
                            text = _("A landscape is currently being generated in the background. Please wait a moment."),
                        })
                    else
                        local started = self:generateNextInBackground(true)
                        if started then
                            UIManager:show(InfoMessage:new{
                                text = _("Generating new Shan Shui landscapes for both orientations (~10s)..."),
                            })
                        else
                            UIManager:show(InfoMessage:new{
                                text = _("Failed to start landscape generator."),
                            })
                        end
                    end
                end,
            },
            {
                text = _("About"),
                callback = function()
                    local count = #self:getLandscapePool()
                    local w, h = self:getTargetDimensions()
                    local portrait_ready = self:getSlotPath(false) ~= nil
                    local landscape_ready = self:getSlotPath(true) ~= nil
                    local info = string.format(
                        _("{Shan, Shui}* Infinite Procedural Landscape\n\n" ..
                          "Procedural Chinese landscape painting screensaver based on LingDong-/shan-shui-inf.\n\n" ..
                          "Target Resolution: %dx%d\n" ..
                          "Transparent Sky: %s\n" ..
                          "Portrait slot: %s\n" ..
                          "Landscape slot: %s\n" ..
                          "Stored Landscapes: %d\n" ..
                          "Location: /mnt/us/koreader/screensavers/shanshui"),
                        w, h,
                        self:isTransparentSky() and _("Enabled") or _("Disabled"),
                        portrait_ready and _("ready") or _("missing"),
                        landscape_ready and _("ready") or _("missing"),
                        count
                    )
                    UIManager:show(InfoMessage:new{
                        text = info,
                    })
                end,
            },
        },
    }
end

return ShanShui
