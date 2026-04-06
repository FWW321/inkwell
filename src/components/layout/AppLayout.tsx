import { useLocation, Outlet } from "react-router-dom";
import { AnimatePresence, motion } from "motion/react";

const AppLayout = () => {
  const { pathname } = useLocation();

  return (
    <div className="flex h-screen flex-col">
      <AnimatePresence mode="wait">
        <motion.div
          key={pathname}
          className="flex flex-1 flex-col overflow-hidden"
          initial={{ opacity: 0 }}
          animate={{ opacity: 1 }}
          exit={{ opacity: 0 }}
          transition={{ duration: 0.15 }}
        >
          <Outlet />
        </motion.div>
      </AnimatePresence>
    </div>
  );
};

export default AppLayout;
