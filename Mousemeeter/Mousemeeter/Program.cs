using System.Diagnostics;
using System.Globalization;
using System.Runtime.InteropServices;
using Timer = System.Windows.Forms.Timer;

namespace Mousemeeter;

public partial class MousemeeterApp : Form
{
    private MousemeeterConfig config;
    private VoicemeeterController vmController;
    private NotifyIcon trayIcon;
    private bool isActivated = true;
    private DateTime lastF24Press = DateTime.MinValue;
    private Timer inputTimer;
    private IntPtr mouseHookId = IntPtr.Zero;
    private WinAPI.LowLevelMouseProc mouseHookProc;
    private MouseStateTracker mouseStateTracker = new MouseStateTracker();

    public MousemeeterApp()
    {
        InitializeComponent();
        InitializeApplication();
    }

    private void InitializeComponent()
    {
        this.WindowState = FormWindowState.Minimized;
        this.ShowInTaskbar = false;
        this.Visible = false;
    }

    private void InitializeApplication()
    {
        config = new MousemeeterConfig();
        config.LoadConfig();

        WaitForVoicemeeter();
        vmController = new VoicemeeterController(config);
        SetupSystemOptimizations();

        if (config.ResetOnStartup)
        {
            vmController.LoadProfile(config.DefaultFile);
            config.CurrentFile = config.DefaultFile;
        }

        SetupTrayIcon();
        SetupInputTimer();
        SetupMouseHook();

        Console.WriteLine("Mousemeeter started successfully");
    }

    private void WaitForVoicemeeter()
    {
        string[] vmProcessNames = { "voicemeeter8", "voicemeeter8x64", "voicemeeter" };

        Console.WriteLine("Waiting for Voicemeeter to start...");
        bool found = false;

        while (!found)
        {
            foreach (string processName in vmProcessNames)
            {
                Process[] processes = Process.GetProcessesByName(processName);
                if (processes.Length > 0)
                {
                    Console.WriteLine($"Found Voicemeeter process: {processName}");
                    found = true;
                    break;
                }
            }

            if (!found)
                Thread.Sleep(1000);
        }

        Thread.Sleep(5000);
    }

    private void SetupSystemOptimizations()
    {
        if (config.SetAffinity)
        {
            try
            {
                Process.GetCurrentProcess().PriorityClass = ProcessPriorityClass.High;
                Console.WriteLine("Set high process priority");
            }
            catch (Exception ex)
            {
                Console.WriteLine($"Failed to set process priority: {ex.Message}");
            }
        }

        if (config.SetCracklingFix)
        {
            try
            {
                var processStartInfo = new ProcessStartInfo
                {
                    FileName = "powershell",
                    Arguments = "$Process = Get-Process audiodg; $Process.ProcessorAffinity=1; $Process.PriorityClass=\"High\"",
                    WindowStyle = ProcessWindowStyle.Hidden,
                    CreateNoWindow = true
                };
                Process.Start(processStartInfo);
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
        trayIcon = new NotifyIcon();
        trayIcon.Icon = new Icon("icon.ico");
        trayIcon.Text = "Mousemeeter";
        trayIcon.Visible = true;

        var contextMenu = new ContextMenuStrip();
        contextMenu.Items.Add("Reload", null, (s, e) => ReloadApplication());
        contextMenu.Items.Add("Refresh Config", null, (s, e) => RefreshConfig());
        contextMenu.Items.Add("-");
        contextMenu.Items.Add("Open Config", null, (s, e) => OpenConfig());
        contextMenu.Items.Add("-");
        contextMenu.Items.Add("Exit", null, (s, e) => ExitApplication());

        trayIcon.ContextMenuStrip = contextMenu;
    }

    private void SetupInputTimer()
    {
        inputTimer = new Timer();
        inputTimer.Interval = 50;
        inputTimer.Tick += InputTimer_Tick;
        inputTimer.Start();
    }

    private void SetupMouseHook()
    {
        mouseHookProc = MouseHookCallback;
        mouseHookId = WinAPI.SetWindowsHookEx(WinAPI.WH_MOUSE_LL, mouseHookProc,
            WinAPI.GetModuleHandle(Process.GetCurrentProcess().MainModule.ModuleName), 0);

        if (mouseHookId == IntPtr.Zero)
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
        if (nCode >= 0 && isActivated)
        {
            try
            {
                var hookStruct = Marshal.PtrToStructure<WinAPI.MSLLHOOKSTRUCT>(lParam);
                MouseEvent mouseEvent = new MouseEvent { Timestamp = DateTime.Now };
                bool queueEvent = false;
                bool blockInput = false;

                switch ((int)wParam)
                {
                    case WinAPI.WM_XBUTTONDOWN:
                        int downButton = unchecked((int)((uint)hookStruct.mouseData >> 16));
                        if (downButton == 1)
                        {
                            mouseEvent.Type = MouseEvent.EventType.XButton1Down;
                            queueEvent = true;
                            blockInput = true;
                        }
                        else if (downButton == 2)
                        {
                            mouseEvent.Type = MouseEvent.EventType.XButton2Down;
                            queueEvent = true;
                            blockInput = true;
                        }
                        break;

                    case WinAPI.WM_XBUTTONUP:
                        int upButton = unchecked((int)((uint)hookStruct.mouseData >> 16));
                        if (upButton == 1)
                        {
                            mouseEvent.Type = MouseEvent.EventType.XButton1Up;
                            queueEvent = true;
                            blockInput = mouseStateTracker.WasVolumeControlUsed;
                        }
                        else if (upButton == 2)
                        {
                            mouseEvent.Type = MouseEvent.EventType.XButton2Up;
                            queueEvent = true;
                            blockInput = mouseStateTracker.WasVolumeControlUsed;
                        }
                        break;

                    case WinAPI.WM_MOUSEWHEEL:
                        if (mouseStateTracker.HotkeyState)
                        {
                            int delta = unchecked((short)((uint)hookStruct.mouseData >> 16));
                            mouseEvent.Type = delta > 0 ? MouseEvent.EventType.WheelUp : MouseEvent.EventType.WheelDown;
                            queueEvent = true;
                            blockInput = true;
                        }
                        break;

                    case WinAPI.WM_LBUTTONDOWN:
                        if (mouseStateTracker.HotkeyState)
                        {
                            mouseEvent.Type = MouseEvent.EventType.LeftDown;
                            queueEvent = true;
                            blockInput = true;
                        }
                        break;

                    case WinAPI.WM_RBUTTONDOWN:
                        if (mouseStateTracker.HotkeyState)
                        {
                            mouseEvent.Type = MouseEvent.EventType.RightDown;
                            queueEvent = true;
                            blockInput = true;
                        }
                        break;

                    case WinAPI.WM_RBUTTONUP:
                        if (mouseStateTracker.HotkeyState)
                        {
                            blockInput = true;
                        }
                        break;

                    case WinAPI.WM_MBUTTONDOWN:
                        if (mouseStateTracker.HotkeyState)
                        {
                            mouseEvent.Type = MouseEvent.EventType.MiddleDown;
                            queueEvent = true;
                            blockInput = true;
                        }
                        break;

                    default:
                        return WinAPI.CallNextHookEx(mouseHookId, nCode, wParam, lParam);
                }

                if (queueEvent)
                {
                    mouseStateTracker.QueueEvent(mouseEvent);
                }

                if (blockInput)
                {
                    return (IntPtr)1;
                }
            }
            catch
            {
                // Ignore all exceptions in hook to prevent blocking
            }
        }

        return WinAPI.CallNextHookEx(mouseHookId, nCode, wParam, lParam);
    }

    private void InputTimer_Tick(object sender, EventArgs e)
    {
        if (!isActivated) return;

        ProcessMouseEvents();

        HandleGlobalHotkeys();
    }

    private void ProcessMouseEvents()
    {
        var events = mouseStateTracker.DequeueEvents();

        foreach (var mouseEvent in events)
        {
            mouseStateTracker.ProcessEvent(mouseEvent);

            switch (mouseEvent.Type)
            {
                case MouseEvent.EventType.WheelUp:
                    if (mouseStateTracker.HotkeyState)
                    {
                        ProcessVolumeUp();
                    }
                    break;

                case MouseEvent.EventType.WheelDown:
                    if (mouseStateTracker.HotkeyState)
                    {
                        ProcessVolumeDown();
                    }
                    break;

                case MouseEvent.EventType.LeftDown:
                    if (mouseStateTracker.XButton2Pressed && !mouseStateTracker.XButton1Pressed)
                    {
                        vmController.QueueMediaAction(VolumeAction.ActionType.MediaPrev);
                    }
                    break;

                case MouseEvent.EventType.RightDown:
                    if (mouseStateTracker.XButton2Pressed && !mouseStateTracker.XButton1Pressed)
                    {
                        vmController.QueueMediaAction(VolumeAction.ActionType.MediaNext);
                    }
                    break;

                case MouseEvent.EventType.MiddleDown:
                    if (mouseStateTracker.XButton2Pressed && !mouseStateTracker.XButton1Pressed)
                    {
                        vmController.QueueMediaAction(VolumeAction.ActionType.MediaPlayPause);
                    }
                    break;
            }
        }
    }

    private void ProcessVolumeUp()
    {
        if (mouseStateTracker.XButton1Pressed && mouseStateTracker.XButton2Pressed)
        {
            vmController.QueueVolumeUp(config.Output3);
        }
        else if (mouseStateTracker.XButton1Pressed)
        {
            vmController.QueueVolumeUp(config.Output1);
        }
        else if (mouseStateTracker.XButton2Pressed)
        {
            vmController.QueueVolumeUp(config.Output2);
        }
    }

    private void ProcessVolumeDown()
    {
        if (mouseStateTracker.XButton1Pressed && mouseStateTracker.XButton2Pressed)
        {
            vmController.QueueVolumeDown(config.Output3);
        }
        else if (mouseStateTracker.XButton1Pressed)
        {
            vmController.QueueVolumeDown(config.Output1);
        }
        else if (mouseStateTracker.XButton2Pressed)
        {
            vmController.QueueVolumeDown(config.Output2);
        }
    }

    private void HandleGlobalHotkeys()
    {
        if ((WinAPI.GetAsyncKeyState(WinAPI.VK_F24) & 0x8000) != 0)
        {
            DateTime now = DateTime.Now;

            if (mouseStateTracker.XButton1Pressed && mouseStateTracker.XButton2Pressed)
            {
                vmController.QueueVolumeMute(config.Output3);
            }
            else if (mouseStateTracker.XButton1Pressed)
            {
                vmController.QueueVolumeMute(config.Output1);
            }
            else if (mouseStateTracker.XButton2Pressed)
            {
                vmController.QueueVolumeMute(config.Output2);
            }
            else
            {
                if ((now - lastF24Press).TotalMilliseconds < 250)
                {
                    if (config.CurrentFile == config.Profile2File)
                    {
                        vmController.LoadProfile(config.DefaultFile);
                        config.CurrentFile = config.DefaultFile;
                    }
                    else
                    {
                        vmController.LoadProfile(config.Profile2File);
                        config.CurrentFile = config.Profile2File;
                    }
                }
                else
                {
                    Task.Delay(250).ContinueWith(t =>
                    {
                        if ((DateTime.Now - lastF24Press).TotalMilliseconds >= 250)
                        {
                            if (config.CurrentFile == config.Profile1File)
                            {
                                vmController.LoadProfile(config.DefaultFile);
                                config.CurrentFile = config.DefaultFile;
                            }
                            else
                            {
                                vmController.LoadProfile(config.Profile1File);
                                config.CurrentFile = config.Profile1File;
                            }
                        }
                    });
                }
                lastF24Press = now;
            }
            Thread.Sleep(100);
        }

        if ((WinAPI.GetAsyncKeyState(WinAPI.VK_F4) & 0x8000) != 0 &&
            (WinAPI.GetAsyncKeyState(WinAPI.VK_CONTROL) & 0x8000) != 0 &&
            (WinAPI.GetAsyncKeyState(WinAPI.VK_MENU) & 0x8000) != 0)
        {
            ForceKillActiveWindow();
            Thread.Sleep(100);
        }

        if ((WinAPI.GetAsyncKeyState(WinAPI.VK_R) & 0x8000) != 0 &&
            (WinAPI.GetAsyncKeyState(WinAPI.VK_CONTROL) & 0x8000) != 0 &&
            (WinAPI.GetAsyncKeyState(WinAPI.VK_SHIFT) & 0x8000) != 0)
        {
            vmController.Restart();
            Thread.Sleep(100);
        }
    }

    protected override void WndProc(ref Message m)
    {
        base.WndProc(ref m);
    }

    private void ForceKillActiveWindow()
    {
        try
        {
            IntPtr hWnd = WinAPI.GetForegroundWindow();
            WinAPI.GetWindowThreadProcessId(hWnd, out uint processId);

            if (processId != 0)
            {
                Process process = Process.GetProcessById((int)processId);
                process.Kill();
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to kill active window: {ex.Message}");
        }
    }

    private void ReloadApplication()
    {
        Application.Restart();
    }

    private void RefreshConfig()
    {
        config.LoadConfig();
        Console.WriteLine("Configuration refreshed");
    }

    private void OpenConfig()
    {
        try
        {
            string configPath = Path.Combine(Application.StartupPath, "config.ini");
            if (File.Exists(configPath))
            {
                Process.Start("notepad.exe", configPath);
            }
        }
        catch (Exception ex)
        {
            Console.WriteLine($"Failed to open config: {ex.Message}");
        }
    }

    private void ExitApplication()
    {
        inputTimer?.Stop();

        if (mouseHookId != IntPtr.Zero)
        {
            WinAPI.UnhookWindowsHookEx(mouseHookId);
            mouseHookId = IntPtr.Zero;
        }

        vmController?.Disconnect();
        trayIcon?.Dispose();
        Application.Exit();
    }

    protected override void Dispose(bool disposing)
    {
        if (disposing)
        {
            inputTimer?.Dispose();

            if (mouseHookId != IntPtr.Zero)
            {
                WinAPI.UnhookWindowsHookEx(mouseHookId);
                mouseHookId = IntPtr.Zero;
            }

            vmController?.Disconnect();
            trayIcon?.Dispose();
        }
        base.Dispose(disposing);
    }
}

public static class Program
{
    [STAThread]
    public static void Main()
    {
        Application.EnableVisualStyles();
        Application.SetCompatibleTextRenderingDefault(false);
        Application.Run(new MousemeeterApp());
    }
}