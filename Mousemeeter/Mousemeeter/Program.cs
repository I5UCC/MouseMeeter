using System.Diagnostics;
using System.Runtime.InteropServices;
using Timer = System.Windows.Forms.Timer;

namespace Mousemeeter;

public sealed partial class MousemeeterApp : IDisposable
{
    private readonly MousemeeterConfig _config = new();
    private readonly MouseStateTracker _mouseStateTracker = new();
    
    private VoicemeeterController? _vmController;
    private NotifyIcon? _trayIcon;
    private Timer? _inputTimer;
    private IntPtr _mouseHookId = IntPtr.Zero;
    private WinAPI.LowLevelMouseProc? _mouseHookProc;
    private ToolStripMenuItem? _toggleTimerMenuItem;
    
    private volatile bool _isActivated = true;
    private volatile bool _isDisposed = false;
    
    private readonly Stopwatch _f24PressStopwatch = Stopwatch.StartNew();
    private const int DoubleClickThresholdMs = 250;
    private const int GlobalHotkeyDelayMs = 100;

    public MousemeeterApp()
    {
        InitializeApplicationAsync();
    }

    private async void InitializeApplicationAsync()
    {
        try
        {
            SetupTrayIcon();
            _config.LoadConfig();

            var voicemeeterPath = _config.VoicemeeterPath ??
                                  Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.ProgramFilesX86),
                                      "VB", "Voicemeeter");
            
            Console.WriteLine($"Voicemeeter path: {voicemeeterPath}");
            
            WinAPI.SetDllDirectory(voicemeeterPath);

            await WaitForVoicemeeterAsync();

            _vmController = new VoicemeeterController(_config);
            SetupSystemOptimizations();

            if (_config.ResetOnStartup)
            {
                _vmController.LoadProfile(_config.DefaultFile);
                _config.CurrentFile = _config.DefaultFile;
            }
            else
            {
                _vmController.SyncAllValues();
            }

            SetupInputTimer();
            SetupMouseHook();

            Console.WriteLine("Mousemeeter started successfully");
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to initialize application: {ex.Message}");
            ExitApplication();
        }
    }

    private static async Task WaitForVoicemeeterAsync()
    {
        string[] vmProcessNames = ["voicemeeter8", "voicemeeter8x64", "voicemeeter"];

        Console.WriteLine("Waiting for Voicemeeter to start...");

        using var cancellationTokenSource = new CancellationTokenSource(TimeSpan.FromMinutes(5)); // 5-minute timeout
        
        try
        {
            while (!cancellationTokenSource.Token.IsCancellationRequested)
            {
                foreach (var processName in vmProcessNames)
                {
                    var processes = Process.GetProcessesByName(processName);
                    try
                    {
                        if (processes.Length > 0)
                        {
                            Console.WriteLine($"Found Voicemeeter process: {processName}");
                            await Task.Delay(2000, cancellationTokenSource.Token); // Startup delay
                            return;
                        }
                    }
                    finally
                    {
                        // Properly dispose all processes
                        foreach (var process in processes)
                        {
                            process?.Dispose();
                        }
                    }
                }

                await Task.Delay(1000, cancellationTokenSource.Token);
            }
        }
        catch (OperationCanceledException)
        {
            throw new TimeoutException("Voicemeeter did not start within the expected time.");
        }
    }

    private void SetupSystemOptimizations()
    {
        if (_config.SetAffinity)
        {
            try
            {
                using var currentProcess = Process.GetCurrentProcess();
                currentProcess.PriorityClass = ProcessPriorityClass.High;
                Console.WriteLine("Set high process priority");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to set process priority: {ex.Message}");
            }
        }

        if (_config.SetCracklingFix)
        {
            try
            {
                using var process = Process.Start(new ProcessStartInfo
                {
                    FileName = "powershell",
                    Arguments = "$Process = Get-Process audiodg -ErrorAction SilentlyContinue; if($Process) { $Process.ProcessorAffinity=1; $Process.PriorityClass='High' }",
                    WindowStyle = ProcessWindowStyle.Hidden,
                    CreateNoWindow = true,
                    UseShellExecute = false
                });
                
                Console.WriteLine("Applied crackling fix");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to apply crackling fix: {ex.Message}");
            }
        }
    }

    private void SetupTrayIcon()
    {
        _trayIcon = new NotifyIcon
        {
            Icon = new Icon("icon.ico"),
            Text = "Mousemeeter",
            Visible = true
        };

        var contextMenu = new ContextMenuStrip();
        _toggleTimerMenuItem = new ToolStripMenuItem("Disable", null, (_, _) => ToggleActivated());
        contextMenu.Items.AddRange([
            _toggleTimerMenuItem,
            new ToolStripSeparator(),
            new ToolStripMenuItem("Reload", null, (_, _) => ReloadApplication()),
            new ToolStripMenuItem("Refresh Config", null, (_, _) => RefreshConfig()),
            new ToolStripSeparator(),
            new ToolStripMenuItem("Open Config", null, (_, _) => OpenConfig()),
            new ToolStripSeparator(),
            new ToolStripMenuItem("Exit", null, (_, _) => ExitApplication())
        ]);

        _trayIcon.ContextMenuStrip = contextMenu;
    }

    private void ToggleActivated()
    {
        _isActivated = !_isActivated;

        if (_toggleTimerMenuItem is not null)
        {
            _toggleTimerMenuItem.Text = _isActivated ? "Disable" : "Enable";
        }

        try
        {
            if (!_isActivated)
                System.Media.SystemSounds.Beep.Play();
        }
        catch
        {
            // ignored
        }
    }

    private void SetupInputTimer()
    {
        _inputTimer = new Timer
        {
            Interval = 100
        };
        _inputTimer.Tick += InputTimer_Tick;
        _inputTimer.Start();
        if (_toggleTimerMenuItem is not null) _toggleTimerMenuItem.Text = "Disable";
    }

    private void SetupMouseHook()
    {
        _mouseHookProc = MouseHookCallback;
        using var currentProcess = Process.GetCurrentProcess();
        
        _mouseHookId = WinAPI.SetWindowsHookEx(
            WinAPI.WH_MOUSE_LL, 
            _mouseHookProc,
            WinAPI.GetModuleHandle(currentProcess.MainModule?.ModuleName), 
            0);

        if (_mouseHookId == IntPtr.Zero)
        {
            Console.WriteLine("Failed to install mouse hook");
        }
        else
        {
            Console.WriteLine("Mouse hook installed successfully");
        }
    }

    private IntPtr MouseHookCallback(int nCode, IntPtr wParam, IntPtr lParam)
    {
        if (nCode < 0 || !_isActivated || _isDisposed)
        {
            return WinAPI.CallNextHookEx(_mouseHookId, nCode, wParam, lParam);
        }

        try
        {
            var hookStruct = Marshal.PtrToStructure<WinAPI.MSLLHOOKSTRUCT>(lParam);
            var mouseEvent = new MouseEvent { Timestamp = DateTime.Now };
            var shouldQueue = false;
            var shouldBlock = false;

            var eventResult = (int)wParam switch
            {
                WinAPI.WM_XBUTTONDOWN => ProcessXButtonDown(hookStruct.mouseData, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                WinAPI.WM_XBUTTONUP => ProcessXButtonUp(hookStruct.mouseData, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                WinAPI.WM_MOUSEWHEEL when _mouseStateTracker.HotkeyState => ProcessMouseWheel(hookStruct.mouseData, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                WinAPI.WM_LBUTTONDOWN when _mouseStateTracker.HotkeyState => ProcessButtonDown(MouseEvent.EventType.LeftDown, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                WinAPI.WM_RBUTTONDOWN when _mouseStateTracker.HotkeyState => ProcessButtonDown(MouseEvent.EventType.RightDown, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                WinAPI.WM_RBUTTONUP when _mouseStateTracker.HotkeyState => (false, true),
                WinAPI.WM_MBUTTONDOWN when _mouseStateTracker.HotkeyState => ProcessButtonDown(MouseEvent.EventType.MiddleDown, ref mouseEvent, ref shouldQueue, ref shouldBlock),
                _ => (false, false)
            };

            shouldQueue = eventResult.Item1;
            shouldBlock = eventResult.Item2;

            if (shouldQueue)
            {
                _mouseStateTracker.QueueEvent(mouseEvent);
            }

            return shouldBlock ? (IntPtr)1 : WinAPI.CallNextHookEx(_mouseHookId, nCode, wParam, lParam);
        }
        catch
        {
            return WinAPI.CallNextHookEx(_mouseHookId, nCode, wParam, lParam);
        }
    }

    private static (bool shouldQueue, bool shouldBlock) ProcessXButtonDown(uint mouseData, ref MouseEvent mouseEvent, ref bool shouldQueue, ref bool shouldBlock)
    {
        var button = unchecked((int)(mouseData >> 16));
        if (button == 1)
        {
            mouseEvent.Type = MouseEvent.EventType.XButton1Down;
            return (true, true);
        }
        if (button == 2)
        {
            mouseEvent.Type = MouseEvent.EventType.XButton2Down;
            return (true, true);
        }
        return (false, false);
    }

    private (bool shouldQueue, bool shouldBlock) ProcessXButtonUp(uint mouseData, ref MouseEvent mouseEvent, ref bool shouldQueue, ref bool shouldBlock)
    {
        var button = unchecked((int)(mouseData >> 16));
        if (button == 1)
        {
            mouseEvent.Type = MouseEvent.EventType.XButton1Up;
            return (true, _mouseStateTracker.WasVolumeControlUsed);
        }
        if (button == 2)
        {
            mouseEvent.Type = MouseEvent.EventType.XButton2Up;
            return (true, _mouseStateTracker.WasVolumeControlUsed);
        }
        return (false, false);
    }

    private static (bool shouldQueue, bool shouldBlock) ProcessMouseWheel(uint mouseData, ref MouseEvent mouseEvent, ref bool shouldQueue, ref bool shouldBlock)
    {
        var delta = unchecked((short)(mouseData >> 16));
        mouseEvent.Type = delta > 0 ? MouseEvent.EventType.WheelUp : MouseEvent.EventType.WheelDown;
        return (true, true);
    }

    private static (bool shouldQueue, bool shouldBlock) ProcessButtonDown(MouseEvent.EventType eventType, ref MouseEvent mouseEvent, ref bool shouldQueue, ref bool shouldBlock)
    {
        mouseEvent.Type = eventType;
        return (true, true);
    }

    private void InputTimer_Tick(object? sender, EventArgs e)
    {
        if (_isDisposed) return;

        HandleGlobalHotkeys();

        if (!_isActivated) return;

        ProcessMouseEvents();
    }

    private void ProcessMouseEvents()
    {
        try
        {
            var events = _mouseStateTracker.DequeueEvents();

            var eventsSpan = CollectionsMarshal.AsSpan(events);
            foreach (ref readonly var mouseEvent in eventsSpan)
            {
                _mouseStateTracker.ProcessEvent(mouseEvent);

                switch (mouseEvent.Type)
                {
                    case MouseEvent.EventType.WheelUp when _mouseStateTracker.HotkeyState:
                        ProcessVolumeUp();
                        break;

                    case MouseEvent.EventType.WheelDown when _mouseStateTracker.HotkeyState:
                        ProcessVolumeDown();
                        break;

                    case MouseEvent.EventType.LeftDown when _mouseStateTracker.XButton2Pressed && !_mouseStateTracker.XButton1Pressed:
                        _vmController?.QueueMediaAction(VolumeAction.ActionType.MediaPrev);
                        break;

                    case MouseEvent.EventType.RightDown when _mouseStateTracker.XButton2Pressed && !_mouseStateTracker.XButton1Pressed:
                        _vmController?.QueueMediaAction(VolumeAction.ActionType.MediaNext);
                        break;

                    case MouseEvent.EventType.MiddleDown when _mouseStateTracker.XButton2Pressed && !_mouseStateTracker.XButton1Pressed:
                        _vmController?.QueueMediaAction(VolumeAction.ActionType.MediaPlayPause);
                        break;
                }
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Error processing mouse events: {ex.Message}");
        }
    }

    private void ProcessVolumeUp()
    {
        if (_vmController is null) return;

        var output = (_mouseStateTracker.XButton1Pressed, _mouseStateTracker.XButton2Pressed) switch
        {
            (true, true) => _config.Output3,
            (true, false) => _config.Output1,
            (false, true) => _config.Output2,
            _ => -1
        };

        if (output >= 0)
        {
            _vmController.QueueVolumeUp(output);
        }
    }

    private void ProcessVolumeDown()
    {
        if (_vmController is null) return;

        var output = (_mouseStateTracker.XButton1Pressed, _mouseStateTracker.XButton2Pressed) switch
        {
            (true, true) => _config.Output3,
            (true, false) => _config.Output1,
            (false, true) => _config.Output2,
            _ => -1
        };

        if (output >= 0)
        {
            _vmController.QueueVolumeDown(output);
        }
    }

    private void HandleGlobalHotkeys()
    {
        if ((WinAPI.GetAsyncKeyState(WinAPI.VK_F24) & 0x8000) != 0)
        {
            ProcessF24Hotkey();
            Thread.Sleep(GlobalHotkeyDelayMs);
        }

        if ((WinAPI.GetAsyncKeyState(WinAPI.VK_F15) & 0x8000) != 0)
        {
            ToggleActivated();
            Thread.Sleep(GlobalHotkeyDelayMs);
        }

        if (IsHotkeyPressed(WinAPI.VK_F4, WinAPI.VK_CONTROL, WinAPI.VK_MENU))
        {
            ForceKillActiveWindow();
            Thread.Sleep(GlobalHotkeyDelayMs);
        }

        if (IsHotkeyPressed(WinAPI.VK_R, WinAPI.VK_CONTROL, WinAPI.VK_SHIFT))
        {
            _vmController?.Restart();
            Thread.Sleep(GlobalHotkeyDelayMs);
        }
    }

    private void ProcessF24Hotkey()
    {
        if (_vmController is null) return;

        var elapsedMs = _f24PressStopwatch.ElapsedMilliseconds;

        var output = (_mouseStateTracker.XButton1Pressed, _mouseStateTracker.XButton2Pressed) switch
        {
            (true, true) => _config.Output3,
            (true, false) => _config.Output1,
            (false, true) => _config.Output2,
            _ => -1
        };

        if (output >= 0)
        {
            _vmController.QueueVolumeMute(output);
            return;
        }

        if (elapsedMs < DoubleClickThresholdMs)
        {
            var targetFile = _config.CurrentFile == _config.Profile2File ? _config.DefaultFile : _config.Profile2File;
            _vmController.LoadProfile(targetFile);
            _config.CurrentFile = targetFile;
        }
        else
        {
            _ = Task.Delay(DoubleClickThresholdMs).ContinueWith(_ =>
            {
                if (_f24PressStopwatch.ElapsedMilliseconds < DoubleClickThresholdMs) return;
                
                var targetFile = _config.CurrentFile == _config.Profile1File ? _config.DefaultFile : _config.Profile1File;
                _vmController?.LoadProfile(targetFile);
                _config.CurrentFile = targetFile;
            }, TaskScheduler.Default);
        }

        _f24PressStopwatch.Restart();
    }

    private static bool IsHotkeyPressed(params int[] keys)
    {
        return keys.All(key => (WinAPI.GetAsyncKeyState(key) & 0x8000) != 0);
    }

    private static void ForceKillActiveWindow()
    {
        try
        {
            var hWnd = WinAPI.GetForegroundWindow();
            WinAPI.GetWindowThreadProcessId(hWnd, out var processId);

            if (processId == 0) return;
            
            using var process = Process.GetProcessById((int)processId);
            process.Kill();
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to kill active window: {ex.Message}");
        }
    }

    private static void ReloadApplication()
    {
        Application.Restart();
    }

    private void RefreshConfig()
    {
        _config.LoadConfig();
        Console.WriteLine("Configuration refreshed");
    }

    private static void OpenConfig()
    {
        try
        {
            var configPath = Path.Combine(Application.StartupPath, "config.ini");
            if (File.Exists(configPath))
            {
                using var process = Process.Start("notepad.exe", configPath);
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to open config: {ex.Message}");
        }
    }

    private void ExitApplication()
    {
        if (_isDisposed) return;

        _isDisposed = true;
        _inputTimer?.Stop();

        if (_mouseHookId != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(_mouseHookId);
            _mouseHookId = IntPtr.Zero;
        }

        _vmController?.Disconnect();
        _trayIcon?.Dispose();
        Application.Exit();
    }

    public void Dispose()
    {
        if (!_isDisposed)
        {
            _isDisposed = true;
            _inputTimer?.Dispose();

            if (_mouseHookId != IntPtr.Zero)
            {
                WinAPI.UnhookWindowsHookEx(_mouseHookId);
                _mouseHookId = IntPtr.Zero;
            }

            _vmController?.Disconnect();
            _trayIcon?.Dispose();
        }
    }
}

public static class Program
{
    [STAThread]
    public static void Main()
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);

        using var app = new MousemeeterApp();
        Application.Run();
    }
}