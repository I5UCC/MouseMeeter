using System;
using System.Collections.Generic;
using System.Linq;
using System.Text;
using System.Threading.Tasks;

namespace Mousemeeter;

public class MouseStateTracker
{
    private volatile bool xButton1Pressed = false;
    private volatile bool xButton2Pressed = false;
    private volatile bool wasVolumeControlUsed = false;
    private readonly Queue<MouseEvent> eventQueue = new Queue<MouseEvent>();
    private readonly object queueLock = new object();

    public bool XButton1Pressed => xButton1Pressed;
    public bool XButton2Pressed => xButton2Pressed;
    public bool HotkeyState => xButton1Pressed || xButton2Pressed;

    public bool WasVolumeControlUsed => wasVolumeControlUsed;

    public void QueueEvent(MouseEvent mouseEvent)
    {
        lock (queueLock)
        {
            eventQueue.Enqueue(mouseEvent);
        }
    }

    public List<MouseEvent> DequeueEvents()
    {
        var events = new List<MouseEvent>();
        lock (queueLock)
        {
            while (eventQueue.Count > 0)
            {
                events.Add(eventQueue.Dequeue());
            }
        }
        return events;
    }

    public bool ProcessEvent(MouseEvent mouseEvent)
    {
        switch (mouseEvent.Type)
        {
            case MouseEvent.EventType.XButton1Down:
                xButton1Pressed = true;
                return false; // Don't block

            case MouseEvent.EventType.XButton1Up:
                bool shouldBlockX1 = wasVolumeControlUsed && xButton1Pressed;
                xButton1Pressed = false;
                wasVolumeControlUsed = false;
                return shouldBlockX1;

            case MouseEvent.EventType.XButton2Down:
                xButton2Pressed = true;
                return false; // Don't block

            case MouseEvent.EventType.XButton2Up:
                bool shouldBlockX2 = wasVolumeControlUsed && xButton2Pressed;
                xButton2Pressed = false;
                wasVolumeControlUsed = false;
                return shouldBlockX2;

            case MouseEvent.EventType.WheelUp:
            case MouseEvent.EventType.WheelDown:
                // Mouse wheel is only captured when hotkey state is active
                if (HotkeyState)
                {
                    wasVolumeControlUsed = true;
                    return true; // Block wheel events when used for volume control
                }
                return false; // Don't block when no hotkey is pressed

            case MouseEvent.EventType.LeftDown:
            case MouseEvent.EventType.RightDown:
            case MouseEvent.EventType.MiddleDown:
                if (xButton2Pressed && !xButton1Pressed)
                {
                    wasVolumeControlUsed = true;
                    return true; // Block media controls
                }
                return false;

            default:
                return false;
        }
    }

    public void SetVolumeControlUsed()
    {
        wasVolumeControlUsed = true;
    }
}

public struct MouseEvent
{
    public enum EventType
    {
        XButton1Down, XButton1Up, XButton2Down, XButton2Up,
        WheelUp, WheelDown,
        LeftDown, RightDown, MiddleDown
    }
    public EventType Type;
    public DateTime Timestamp;
}