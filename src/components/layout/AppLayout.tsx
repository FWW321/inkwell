import { Outlet } from "react-router-dom";

const AppLayout = () => {
  return (
    <div className="flex h-screen flex-col">
      <Outlet />
    </div>
  );
};

export default AppLayout;
