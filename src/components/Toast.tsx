import { ReactNode } from "react";
import { assertExhaustive } from "../utils";
import { ToastContainer, toast } from "react-toastify";
import "react-toastify/dist/ReactToastify.css";

type MessageType = "info" | "warn" | "error";

export function createToast(
  type: MessageType,
  title: string,
  message?: string
) {
  const messageCombo = message ? `${title}\n${message}` : title;

  switch (type) {
    case "info":
      toast.info(messageCombo, { autoClose: 1500 });
      break;
    case "warn":
      toast.warn(messageCombo, { autoClose: 3000 });
      break;
    case "error":
      toast.error(messageCombo, { autoClose: 5000 });
      break;
    default:
      assertExhaustive(type);
  }
}

export function ToastProvider({ children }: { children: ReactNode }) {
  return (
    <>
      {children}
      <ToastContainer
        position="top-right"
        hideProgressBar={false}
        newestOnTop={false}
        closeOnClick
        rtl={false}
        pauseOnFocusLoss
        draggable
        pauseOnHover
        theme="dark"
      />
    </>
  );
}
