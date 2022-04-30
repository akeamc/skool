import {
  FunctionComponent,
  useCallback,
  useEffect,
  useRef,
  useState,
} from "react";
import Link from "next/link";
import styles from "./layout.module.scss";
import classNames from "classnames/bind";
import { useAuth } from "../../lib/auth";

const cx = classNames.bind(styles);

const NavbarItem: FunctionComponent<{ href: string }> = ({
  href,
  children,
}) => {
  return (
    <li>
      <Link href={href}>
        <a>{children}</a>
      </Link>
    </li>
  );
};

const Navbar: FunctionComponent = () => {
  const [floating, setFloating] = useState(false);
  const { authenticated, logout } = useAuth();

  const onScroll = useCallback((event: Event) => {
    setFloating(window.scrollY > 0);
  }, []);

  useEffect(() => {
    document.addEventListener("scroll", onScroll);

    return () => document.removeEventListener("scroll", onScroll);
  }, [onScroll]);

  return (
    <nav className={cx("navbar", { floating })}>
      <ul>
        <NavbarItem href="/">Start</NavbarItem>
        <NavbarItem href="/schedule">Schema</NavbarItem>
        {authenticated && <button onClick={logout}>Logga ut</button>}
      </ul>
    </nav>
  );
};

const Layout: FunctionComponent<{ padTop?: boolean, navbar?: boolean, }> = ({
  children,
  navbar = true,
  padTop = navbar,
}) => (
  <div className={cx("layout", { padTop })}>
    {navbar &&
      <Navbar />}
    <main>{children}</main>
  </div>
);

export default Layout;
