use jwt_auth_lib::util::validation::*;

#[cfg(test)]
mod username {
    use super::*;

    #[test]
    fn test_length() {
        assert_eq!(username(&"myusernameblablabla123456789".to_string()).0, false);
        assert_eq!(username(&"me".to_string()).0,                           false);
        assert_eq!(username(&"myusername1234".to_string()).0,               true);
        assert_eq!(username(&"user1".to_string()).0,                        true);
    }

    #[test]
    fn test_chars() {
        assert_eq!(username(&"myuser@#?!\"=§".to_string()).0, false);
        assert_eq!(username(&"字漢字".to_string()).0,          false);
        assert_eq!(username(&"🥸😂".to_string()).0,           false);
        assert_eq!(username(&"My_Username12".to_string()).0,  true);
        assert_eq!(username(&"User.name".to_string()).0,      true);
        assert_eq!(username(&"user-name".to_string()).0,      true);
    }
}


#[cfg(test)]
mod password {
    use super::*;

    #[test]
    fn test_length() {
        assert_eq!(password(&"Password*123456789abcdefghiklmnopqrstuvwxyz".to_string()).0, false);
        assert_eq!(password(&"Psw1*".to_string()).0,                                       false);
        assert_eq!(password(&"MyPassword123*#".to_string()).0,                             true);
    }

    #[test]
    fn test_chars() {
        assert_eq!(password(&"mypassword1*".to_string()).0, false);
        assert_eq!(password(&"MYPASSWORD2*".to_string()).0, false);
        assert_eq!(password(&"MyPassword*#".to_string()).0, false);
        assert_eq!(password(&"MyPassword1234".to_string()).0, false);

        assert_eq!(password(&"MyPassword1234*#".to_string()).0, true);
    }
}