import classNames from "classnames";
import { AnimatePresence, motion } from "framer-motion";
import {
  createContext,
  useContext,
  useEffect,
  ReactNode,
  useState,
  useCallback,
} from "react";
import { assertExhaustive } from "../utils";

type Toast = {
  title: string;
  message: string;
  type: MessageType;
  date: number;
  guid: string;
};
type MessageType = "info" | "warn" | "error";

type ToastContextProps = {
  createToast: (title: string, message: string, type: MessageType) => void;
};
const ToastContext = createContext<ToastContextProps>({
  createToast: () => null,
});

export function useToastContext() {
  return useContext(ToastContext);
}

function toastDuraction(t: Toast) {
  switch (t.type) {
    case "info":
      return 1000;
    case "warn":
      return 3000;
    case "error":
      return 5000;
    default:
      assertExhaustive(t.type);
  }
}

function guid() {
  // Guid because we could have errors come in with same attributes at the same time
  function _p8(s?: boolean) {
    const p = (Math.random().toString(16) + "000000000").substring(2, 8);
    return s ? "-" + p.substring(0, 4) + "-" + p.substring(4, 4) : p;
  }
  return _p8() + _p8(true) + _p8(true) + _p8();
}

export function ToastProvider({ children }: { children: ReactNode }) {
  const [toasts, setToasts] = useState<Toast[]>([]);

  useEffect(() => {
    if (toasts.length == 0) return;

    const minTimeToWait = toasts
      .map((t) => t.date - Date.now() + toastDuraction(t))
      .reduce((prv, val) => Math.min(prv, val), Infinity);
    const EXTRA_WAIT_TIME = 50;
    const timer = setTimeout(() => {
      setToasts((prevToasts) =>
        prevToasts.filter((t) => Date.now() - t.date < toastDuraction(t))
      );
    }, EXTRA_WAIT_TIME + minTimeToWait);

    return () => clearTimeout(timer);
  }, [toasts]);

  const createToast = useCallback(
    (title: string, message: string, type: MessageType) => {
      setToasts((prevToasts) => [
        ...prevToasts,
        { title, message, type, date: Date.now(), guid: guid() },
      ]);
    },
    [setToasts]
  );

  return (
    <ToastContext.Provider value={{ createToast }}>
      {children}
      <div className="flex flex-col-reverse w-screen h-screen top-0 right-0 fixed pointer-events-none">
        <AnimatePresence>
          {toasts.map((toast) => (
            <motion.div
              key={toast.guid}
              className={classNames(
                toast.type == "info" && "bg-blue-400",
                toast.type == "warn" && "bg-amber-400",
                toast.type == "error" && "bg-red-400",
                "w-64 rounded-md m-1 p-1"
              )}
              initial={{ x: "100vh", opacity: 0 }}
              animate={{
                x: "calc(100vw - 300px)",
                opacity: 1,
                transition: {
                  type: "spring",
                  duration: 0.5,
                },
              }}
              exit={{
                x: "100vh",
                opacity: 0,
                transition: { opactity: { duraction: 0.1 } },
              }}
            >
              <div className="font-bold">{toast.title}</div>
              <div className="text-sm whitespace-break-spaces">
                {toast.message}
              </div>
            </motion.div>
          ))}
        </AnimatePresence>
      </div>
    </ToastContext.Provider>
  );
}
