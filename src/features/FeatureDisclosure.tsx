import { ChevronUpIcon } from "@heroicons/react/24/solid";
import classNames from "classnames";
import { AnimatePresence, motion } from "framer-motion";
import { ReactNode, useState } from "react";

export function FeatureDisclosure({
  title,
  titleIsOn = true,
  outline,
  children,
}: {
  title: string;
  titleIsOn?: boolean;
  outline: string;
  children: ReactNode;
}) {
  const [isOpen, setIsOpen] = useState(false);
  return (
    <>
      <button
        className={classNames(
          "flex justify-between items-center px-4 py-2 outline outline-1 rounded-md",
          outline
        )}
        onClick={() => setIsOpen((b) => !b)}
      >
        <div className="flex items-center">
          <div className={titleIsOn ? "text-zinc-100" : "text-zinc-600"}>
            {title}
          </div>
        </div>
        <motion.div animate={{ rotate: isOpen ? 180 : 0 }}>
          <ChevronUpIcon className="h-6 w-6 text-zinc-400" />
        </motion.div>
      </button>
      <AnimatePresence>
        {isOpen && (
          <motion.div
            initial={{ height: 0, opacity: 0 }}
            exit={{
              height: 0,
              opacity: 0,
              transition: {
                opacity: { duration: 0.15 },
                height: { delay: 0.05 },
              },
            }}
            animate={{
              height: "auto",
              opacity: 1,
              transition: {
                type: "spring",
                bounce: 0.3,
                duration: 0.4,
                opacity: { delay: 0.1 },
              },
            }}
          >
            {children}
          </motion.div>
        )}
      </AnimatePresence>
    </>
  );
}
